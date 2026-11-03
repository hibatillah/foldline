//! GPU load from the per-engine counters Task Manager shows, named through DXGI; open only while the page is shown.

use windows::{
    Win32::{
        Graphics::Dxgi::{CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, IDXGIFactory1},
        System::Performance::{
            PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA,
            PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
        },
    },
    core::PCWSTR,
};

use super::sampler::Meter;
use crate::log;

const ENGINE_COUNTER: &str = "\\GPU Engine(*)\\Utilization Percentage";
const ERROR_SUCCESS: u32 = 0;

struct Adapter {
    /// `luid_0x<high>_0x<low>` in lower case, the way the counter instances name an adapter.
    luid: String,
    name: String,
}

pub struct GpuCounters {
    query: PDH_HQUERY,
    engines: PDH_HCOUNTER,
    adapters: Vec<Adapter>,
}

impl GpuCounters {
    /// `None` without hardware adapters or when the counters are unavailable.
    pub fn open() -> Option<Self> {
        let adapters = hardware_adapters();
        if adapters.is_empty() {
            return None;
        }
        let mut query = PDH_HQUERY::default();
        if unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut query) } != ERROR_SUCCESS {
            log::debug("could not open the GPU counter query");
            return None;
        }
        let mut engines = PDH_HCOUNTER::default();
        let path = windows::core::HSTRING::from(ENGINE_COUNTER);
        if unsafe { PdhAddEnglishCounterW(query, &path, 0, &mut engines) } != ERROR_SUCCESS {
            log::debug("could not add the GPU engine counter");
            unsafe { PdhCloseQuery(query) };
            return None;
        }
        // A rate counter needs a first collection before it can report anything.
        unsafe { PdhCollectQueryData(query) };
        Some(Self { query, engines, adapters })
    }

    /// One meter per adapter, in DXGI order; `percent` is `None` until two collections exist.
    pub fn read(&mut self) -> Vec<Meter> {
        let instances =
            if unsafe { PdhCollectQueryData(self.query) } == ERROR_SUCCESS { self.engine_values() } else { Vec::new() };
        let have_values = !instances.is_empty();
        self.adapters
            .iter()
            .map(|adapter| Meter {
                name: adapter.name.clone(),
                percent: have_values.then(|| adapter_load(&adapter.luid, &instances)),
            })
            .collect()
    }

    fn engine_values(&self) -> Vec<(String, f64)> {
        let (mut buffer_size, mut item_count) = (0u32, 0u32);
        let status = unsafe {
            PdhGetFormattedCounterArrayW(self.engines, PDH_FMT_DOUBLE, &mut buffer_size, &mut item_count, None)
        };
        if status != PDH_MORE_DATA || buffer_size == 0 {
            return Vec::new();
        }
        let item_size = size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>();
        // The buffer holds the items followed by the instance names they point into.
        let mut buffer = vec![0u64; (buffer_size as usize).div_ceil(size_of::<u64>())];
        let items = buffer.as_mut_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
        let status = unsafe {
            PdhGetFormattedCounterArrayW(self.engines, PDH_FMT_DOUBLE, &mut buffer_size, &mut item_count, Some(items))
        };
        if status != ERROR_SUCCESS || (item_count as usize) * item_size > buffer.len() * size_of::<u64>() {
            return Vec::new();
        }
        // SAFETY: PDH filled `item_count` items at the start of `buffer`, which stays alive for this loop.
        let items = unsafe { std::slice::from_raw_parts(items, item_count as usize) };
        items
            .iter()
            .filter(|item| item.FmtValue.CStatus == ERROR_SUCCESS)
            .filter_map(|item| {
                let name = unsafe { item.szName.to_string() }.ok()?;
                Some((name.to_lowercase(), unsafe { item.FmtValue.Anonymous.doubleValue }))
            })
            .collect()
    }
}

impl Drop for GpuCounters {
    fn drop(&mut self) {
        unsafe { PdhCloseQuery(self.query) };
    }
}

/// The busiest engine type of one adapter, summed over the processes using it, as Task Manager shows it.
pub fn adapter_load(luid: &str, instances: &[(String, f64)]) -> f32 {
    let mut by_engine_type: Vec<(&str, f64)> = Vec::new();
    for (instance, value) in instances {
        if !instance.contains(luid) {
            continue;
        }
        let engine_type = instance.rsplit_once("engtype_").map_or("", |(_, engine_type)| engine_type);
        match by_engine_type.iter_mut().find(|(known, _)| *known == engine_type) {
            Some((_, total)) => *total += value,
            None => by_engine_type.push((engine_type, *value)),
        }
    }
    by_engine_type.iter().map(|(_, total)| *total).fold(0.0, f64::max).clamp(0.0, 100.0) as f32
}

/// "NVIDIA GeForce RTX 4050 Laptop GPU" becomes "RTX 4050", "AMD Radeon 760M Graphics" becomes "Radeon 760M".
pub fn short_name(description: &str) -> String {
    const DROPPED: [&str; 9] = ["NVIDIA", "GeForce", "AMD", "Intel", "Laptop", "GPU", "Graphics", "(R)", "(TM)"];
    let cleaned = description.replace("(R)", " ").replace("(TM)", " ");
    let words: Vec<&str> = cleaned.split_whitespace().filter(|word| !DROPPED.contains(word)).collect();
    if words.is_empty() { description.trim().to_owned() } else { words.join(" ") }
}

fn hardware_adapters() -> Vec<Adapter> {
    let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else {
        log::debug("could not create a DXGI factory");
        return Vec::new();
    };
    let mut adapters = Vec::new();
    for index in 0.. {
        let Ok(adapter) = (unsafe { factory.EnumAdapters1(index) }) else { break };
        let Ok(description) = (unsafe { adapter.GetDesc1() }) else { continue };
        if description.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
            continue;
        }
        let length =
            description.Description.iter().position(|&unit| unit == 0).unwrap_or(description.Description.len());
        let name = String::from_utf16_lossy(&description.Description[..length]);
        let luid = description.AdapterLuid;
        adapters.push(Adapter {
            luid: format!("luid_0x{:08x}_0x{:08x}", luid.HighPart as u32, luid.LowPart),
            name: short_name(&name),
        });
    }
    adapters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_names_drop_the_vendor_and_marketing_words() {
        assert_eq!(short_name("NVIDIA GeForce RTX 4050 Laptop GPU"), "RTX 4050");
        assert_eq!(short_name("AMD Radeon 760M Graphics"), "Radeon 760M");
        assert_eq!(short_name("Intel(R) UHD Graphics 770"), "UHD 770");
        assert_eq!(short_name("GPU"), "GPU");
    }

    #[test]
    fn an_adapter_load_is_its_busiest_engine_type_summed_over_processes() {
        let luid = "luid_0x00000000_0x0001ee1d";
        let instances = vec![
            (format!("pid_10_{luid}_phys_0_eng_0_engtype_3d"), 20.0),
            (format!("pid_11_{luid}_phys_0_eng_0_engtype_3d"), 15.0),
            (format!("pid_10_{luid}_phys_0_eng_3_engtype_videodecode"), 30.0),
            ("pid_12_luid_0x00000000_0x0000aaaa_phys_0_eng_0_engtype_3d".to_owned(), 90.0),
        ];
        assert_eq!(adapter_load(luid, &instances), 35.0);
        assert_eq!(adapter_load("luid_0x00000000_0x0000ffff", &instances), 0.0);
    }

    #[test]
    fn an_adapter_load_never_passes_one_hundred() {
        let luid = "luid_0x00000000_0x00000001";
        let instances = vec![
            (format!("pid_1_{luid}_phys_0_eng_0_engtype_3d"), 80.0),
            (format!("pid_2_{luid}_phys_0_eng_0_engtype_3d"), 70.0),
        ];
        assert_eq!(adapter_load(luid, &instances), 100.0);
    }
}
