//! The device tree through cfgmgr32: present HID interfaces, a node's ancestors and its string and flag properties.

use windows::{
    Win32::{
        Devices::{
            DeviceAndDriverInstallation::{
                CM_GET_DEVICE_INTERFACE_LIST_PRESENT, CM_Get_DevNode_PropertyW, CM_Get_Device_IDW,
                CM_Get_Device_Interface_List_SizeW, CM_Get_Device_Interface_ListW, CM_Get_Device_Interface_PropertyW,
                CM_Get_Parent, CM_LOCATE_DEVNODE_NORMAL, CM_Locate_DevNodeW, CR_SUCCESS, MAX_DEVICE_ID_LEN,
            },
            HumanInterfaceDevice::HidD_GetHidGuid,
            Properties::{DEVPKEY_Device_InstanceId, DEVPROP_TYPE_BOOLEAN, DEVPROP_TYPE_STRING, DEVPROPTYPE},
        },
        Foundation::DEVPROPKEY,
    },
    core::{HSTRING, PCWSTR},
};

/// Root, bus, hub, interface and HID node fit well within this; the walk stops at the root anyway.
const DEEPEST_ANCESTRY: usize = 12;

/// One node of the device tree.
#[derive(Clone, Debug)]
pub(super) struct Node {
    pub handle: u32,
    pub instance_id: String,
}

/// The paths of every present HID interface, one per top-level collection.
pub(super) fn hid_interface_paths() -> Vec<String> {
    let guid = unsafe { HidD_GetHidGuid() };
    let mut length = 0u32;
    if unsafe {
        CM_Get_Device_Interface_List_SizeW(&mut length, &guid, PCWSTR::null(), CM_GET_DEVICE_INTERFACE_LIST_PRESENT)
    } != CR_SUCCESS
    {
        return Vec::new();
    }
    let mut buffer = vec![0u16; length as usize];
    if unsafe {
        CM_Get_Device_Interface_ListW(&guid, PCWSTR::null(), &mut buffer, CM_GET_DEVICE_INTERFACE_LIST_PRESENT)
    } != CR_SUCCESS
    {
        return Vec::new();
    }
    buffer.split(|&unit| unit == 0).filter(|path| !path.is_empty()).map(String::from_utf16_lossy).collect()
}

/// The node behind an interface path, then its parents up to the root.
pub(super) fn ancestry_of_interface(interface_path: &str) -> Vec<Node> {
    let Some(instance_id) = interface_string_property(interface_path, &DEVPKEY_Device_InstanceId) else {
        return Vec::new();
    };
    let mut handle = 0u32;
    if unsafe { CM_Locate_DevNodeW(&mut handle, &HSTRING::from(instance_id.as_str()), CM_LOCATE_DEVNODE_NORMAL) }
        != CR_SUCCESS
    {
        return Vec::new();
    }
    let mut chain = vec![Node { handle, instance_id }];
    while chain.len() < DEEPEST_ANCESTRY {
        let mut parent = 0u32;
        if unsafe { CM_Get_Parent(&mut parent, handle, 0) } != CR_SUCCESS {
            break;
        }
        let Some(instance_id) = node_instance_id(parent) else { break };
        chain.push(Node { handle: parent, instance_id });
        handle = parent;
    }
    chain
}

fn node_instance_id(handle: u32) -> Option<String> {
    let mut buffer = [0u16; MAX_DEVICE_ID_LEN as usize + 1];
    if unsafe { CM_Get_Device_IDW(handle, &mut buffer, 0) } != CR_SUCCESS {
        return None;
    }
    let length = buffer.iter().position(|&unit| unit == 0).unwrap_or(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..length]))
}

pub(super) fn string_property(handle: u32, key: &DEVPROPKEY) -> Option<String> {
    let bytes = node_property(handle, key, DEVPROP_TYPE_STRING)?;
    Some(utf16_bytes_to_string(&bytes))
}

pub(super) fn flag_property(handle: u32, key: &DEVPROPKEY) -> Option<bool> {
    let bytes = node_property(handle, key, DEVPROP_TYPE_BOOLEAN)?;
    bytes.first().map(|&value| value != 0)
}

fn node_property(handle: u32, key: &DEVPROPKEY, expected: DEVPROPTYPE) -> Option<Vec<u8>> {
    let mut property_type = DEVPROPTYPE::default();
    let mut size = 0u32;
    // The first call only reports the size and fails with CR_BUFFER_SMALL.
    let _ = unsafe { CM_Get_DevNode_PropertyW(handle, key, &mut property_type, None, &mut size, 0) };
    if size == 0 {
        return None;
    }
    let mut bytes = vec![0u8; size as usize];
    if unsafe { CM_Get_DevNode_PropertyW(handle, key, &mut property_type, Some(bytes.as_mut_ptr()), &mut size, 0) }
        != CR_SUCCESS
        || property_type != expected
    {
        return None;
    }
    Some(bytes)
}

fn interface_string_property(interface_path: &str, key: &DEVPROPKEY) -> Option<String> {
    let path = HSTRING::from(interface_path);
    let mut property_type = DEVPROPTYPE::default();
    let mut size = 0u32;
    let _ = unsafe { CM_Get_Device_Interface_PropertyW(&path, key, &mut property_type, None, &mut size, 0) };
    if size == 0 {
        return None;
    }
    let mut bytes = vec![0u8; size as usize];
    if unsafe {
        CM_Get_Device_Interface_PropertyW(&path, key, &mut property_type, Some(bytes.as_mut_ptr()), &mut size, 0)
    } != CR_SUCCESS
        || property_type != DEVPROP_TYPE_STRING
    {
        return None;
    }
    Some(utf16_bytes_to_string(&bytes))
}

fn utf16_bytes_to_string(bytes: &[u8]) -> String {
    let units: Vec<u16> =
        bytes.as_chunks::<2>().0.iter().map(|&pair| u16::from_le_bytes(pair)).take_while(|&unit| unit != 0).collect();
    String::from_utf16_lossy(&units)
}
