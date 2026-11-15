use super::{
    ALERT_TIMER, MAX_ALERT_WAIT_MILLISECONDS, RAIN_ALERT_KEY, RAIN_ALERT_LEAD_MINUTES, RAIN_WINDOW_HOURS,
    REFRESH_MINUTES, REFRESH_TIMER, RETRY_MINUTES, WeatherPage,
    alert::{Rain, body_text},
    fetch::{self, Failure, place_key},
    forecast::Forecast,
};
use crate::{log, page::Context};
use places::Location;
use salah::{Duration, Local};
use std::sync::Arc;

impl WeatherPage {
    pub(super) fn read_settings(&mut self, context: &Context) {
        let read = |key: &str| context.settings.get_string(key).map(str::to_owned);
        self.location = Location::read(&read);
        self.rain_alert = context.settings.get_bool(RAIN_ALERT_KEY, true);
    }

    pub(super) fn start_fetch(&self, context: &Context) {
        if let Some(location) = &self.location {
            fetch::start(location.latitude, location.longitude, Arc::clone(&self.outcome), context.notifier());
        }
    }

    /// Takes what the request found: a forecast, or "offline" with a sooner retry.
    pub(super) fn apply_outcome(&mut self, context: &mut Context) {
        let Some((place, result)) = self.outcome.lock().ok().and_then(|mut slot| slot.take()) else { return };
        let expected = self.location.as_ref().map(|location| place_key(location.latitude, location.longitude));
        if expected != Some(place) {
            return;
        }
        let parsed = match result {
            Ok(body) => {
                let parsed = Forecast::parse(&body, self.now());
                if parsed.is_none() {
                    log::debug(&format!("weather: the response is not a forecast ({} bytes)", body.len()));
                }
                self.is_service_error = parsed.is_none();
                parsed
            }
            Err(failure) => {
                log::debug(&format!("weather: request failed: {}", failure.reason()));
                self.is_service_error = matches!(failure, Failure::Service(_));
                None
            }
        };
        self.is_offline = parsed.is_none();
        if let Some(forecast) = parsed {
            self.forecast = Some(forecast);
        }
        self.refresh_message_width(context.fonts());
        let minutes = if self.is_offline { RETRY_MINUTES } else { REFRESH_MINUTES };
        context.set_timer(REFRESH_TIMER, minutes * 60 * 1000);
        self.arm_alert(context);
        context.request_repaint();
    }

    /// Shows the rain message when rain is under the lead time away, or waits until it is.
    pub(super) fn arm_alert(&mut self, context: &mut Context) {
        context.kill_timer(ALERT_TIMER);
        let now = self.now();
        let Some(hour) = self
            .forecast
            .as_ref()
            .filter(|_| self.is_rain_alert_on())
            .and_then(|forecast| forecast.upcoming_rain(now, RAIN_WINDOW_HOURS))
        else {
            return;
        };
        if self.alerted_for == Some(hour.time) {
            return;
        }
        let due = hour.time - Duration::minutes(RAIN_ALERT_LEAD_MINUTES);
        if due <= now {
            let minutes = (hour.time - now).num_minutes().max(1);
            let time = hour.time.with_timezone(&Local).format("%H:%M").to_string();
            let Some(location) = &self.location else { return };
            let rain = Rain {
                at: hour.time,
                symbol: hour.symbol,
                title: format!("Rain in {minutes} min"),
                body: body_text(&hour.symbol.label(), &time, &location.label),
            };
            self.alerted_for = Some(hour.time);
            self.alert.show(context, rain, now);
        } else {
            let wait = (due - now).num_milliseconds().clamp(1000, MAX_ALERT_WAIT_MILLISECONDS);
            context.set_timer(ALERT_TIMER, wait as u32);
        }
    }
}
