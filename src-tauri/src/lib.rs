use chrono::{ DateTime, Local, NaiveTime, TimeZone };
use chrono_tz::Tz;
use std::str::FromStr;
use reqwest;
use serde::{ Deserialize, Serialize };
use std::collections::HashMap;
use std::time::Duration;
use tauri::{ AppHandle, Emitter, WindowEvent, Manager };
use std::sync::Mutex;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
#[cfg(target_os = "windows")]
use winapi::um::timezoneapi::{ GetTimeZoneInformation, TIME_ZONE_INFORMATION };
#[cfg(target_os = "windows")]
use winapi::um::winnt::{ TIME_ZONE_ID_STANDARD, TIME_ZONE_ID_DAYLIGHT, TIME_ZONE_ID_UNKNOWN };
// Notification functionality will be implemented using system notifications
use tokio::time::interval;
use auto_launch::AutoLaunch;
use tauri::{
    menu::{ Menu, MenuItem },
    tray::{ MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent },
    State,
};

// Global state for settings
type AppState = Mutex<AppSettings>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub timezone: String,
    pub location: String,
}

// Helper function to convert Windows wide string to Rust string
#[cfg(target_os = "windows")]
fn wide_string_to_string(wide_str: &[u16]) -> String {
    let os_string = OsString::from_wide(wide_str);
    os_string.to_string_lossy().to_string()
}

// Get Windows system timezone
#[cfg(target_os = "windows")]
fn get_windows_timezone() -> Result<String, String> {
    unsafe {
        let mut tzi: TIME_ZONE_INFORMATION = std::mem::zeroed();
        let result = GetTimeZoneInformation(&mut tzi);

        match result {
            TIME_ZONE_ID_STANDARD | TIME_ZONE_ID_DAYLIGHT => {
                // Get the standard time zone name
                let tz_name = wide_string_to_string(&tzi.StandardName);
                Ok(tz_name)
            }
            TIME_ZONE_ID_UNKNOWN => { Err("Failed to get timezone information".to_string()) }
            _ => { Err("Unknown timezone result".to_string()) }
        }
    }
}

// Get Windows system location (using timezone as a proxy)
#[cfg(target_os = "windows")]
fn get_windows_location() -> Result<String, String> {
    let timezone = get_windows_timezone()?;

    // Map Windows timezone names to common city names
    let location = match timezone.as_str() {
        "Taipei Standard Time" => "Taipei".to_string(),
        "SE Asia Standard Time" => "Jakarta".to_string(),
        "Tokyo Standard Time" => "Tokyo".to_string(),
        "Singapore Standard Time" => "Singapore".to_string(),
        "Malay Peninsula Standard Time" => "Kuala Lumpur".to_string(),
        "Korea Standard Time" => "Seoul".to_string(),
        "China Standard Time" => "Hong Kong".to_string(),
        "Arabian Standard Time" => "Dubai".to_string(),
        "Arab Standard Time" => "Riyadh".to_string(),
        "Egypt Standard Time" => "Cairo".to_string(),
        "Turkey Standard Time" => "Istanbul".to_string(),
        "GMT Standard Time" => "London".to_string(),
        "Romance Standard Time" => "Paris".to_string(),
        "W. Europe Standard Time" => "Berlin".to_string(),
        "Eastern Standard Time" => "New York".to_string(),
        "Pacific Standard Time" => "Los Angeles".to_string(),
        "Central Standard Time" => "Chicago".to_string(),
        "AUS Eastern Standard Time" => "Sydney".to_string(),
        _ => {
            // Try to extract city name from timezone string
            if timezone.contains("Standard Time") {
                timezone.replace(" Standard Time", "").replace(" Daylight Time", "")
            } else {
                timezone
            }
        }
    };

    Ok(location)
}

// Fallback for non-Windows systems
#[cfg(not(target_os = "windows"))]
fn get_windows_timezone() -> Result<String, String> {
    Err("Windows timezone detection not available on this platform".to_string())
}

#[cfg(not(target_os = "windows"))]
fn get_windows_location() -> Result<String, String> {
    Err("Windows location detection not available on this platform".to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrayerTime {
    pub name: String,
    pub time: String,
    pub datetime: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrayerTimesResponse {
    pub data: PrayerData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrayerData {
    pub timings: HashMap<String, String>,
    pub date: PrayerDate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrayerDate {
    pub readable: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub location: String,
    pub play_sound: bool,
    pub enabled_prayers: Vec<String>,
    pub run_at_startup: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            location: "New Taipei City".to_string(),
            play_sound: true,
            enabled_prayers: vec![
                "Fajr".to_string(),
                "Dhuhr".to_string(),
                "Asr".to_string(),
                "Maghrib".to_string(),
                "Isha".to_string()
            ],
            run_at_startup: false,
        }
    }
}

// Helper function to get timezone from location
fn get_timezone_for_location(location: &str) -> String {
    // Try to parse location as timezone first
    if let Ok(_tz) = Tz::from_str(location) {
        return location.to_string();
    }

    // Try to map common city names to timezones
    let location_lower = location.to_lowercase();
    match location_lower.as_str() {
        location if location.contains("taipei") || location.contains("taiwan") =>
            "Asia/Taipei".to_string(),
        location if location.contains("jakarta") || location.contains("indonesia") =>
            "Asia/Jakarta".to_string(),
        location if location.contains("tokyo") || location.contains("japan") =>
            "Asia/Tokyo".to_string(),
        location if location.contains("singapore") => "Asia/Singapore".to_string(),
        location if location.contains("kuala lumpur") || location.contains("malaysia") =>
            "Asia/Kuala_Lumpur".to_string(),
        location if location.contains("bangkok") || location.contains("thailand") =>
            "Asia/Bangkok".to_string(),
        location if location.contains("manila") || location.contains("philippines") =>
            "Asia/Manila".to_string(),
        location if location.contains("seoul") || location.contains("korea") =>
            "Asia/Seoul".to_string(),
        location if location.contains("hong kong") => "Asia/Hong_Kong".to_string(),
        location if location.contains("dubai") || location.contains("uae") =>
            "Asia/Dubai".to_string(),
        location if location.contains("riyadh") || location.contains("saudi") =>
            "Asia/Riyadh".to_string(),
        location if location.contains("cairo") || location.contains("egypt") =>
            "Africa/Cairo".to_string(),
        location if location.contains("istanbul") || location.contains("turkey") =>
            "Europe/Istanbul".to_string(),
        location if location.contains("london") || location.contains("uk") =>
            "Europe/London".to_string(),
        location if location.contains("paris") || location.contains("france") =>
            "Europe/Paris".to_string(),
        location if location.contains("berlin") || location.contains("germany") =>
            "Europe/Berlin".to_string(),
        location if location.contains("new york") || location.contains("nyc") =>
            "America/New_York".to_string(),
        location if location.contains("los angeles") || location.contains("la") =>
            "America/Los_Angeles".to_string(),
        location if location.contains("chicago") => "America/Chicago".to_string(),
        location if location.contains("toronto") || location.contains("canada") =>
            "America/Toronto".to_string(),
        location if location.contains("sydney") || location.contains("australia") =>
            "Australia/Sydney".to_string(),
        location if location.contains("melbourne") => "Australia/Melbourne".to_string(),
        _ => {
            // Fallback to system timezone
            let offset_seconds = (*Local::now().offset()).local_minus_utc();
            match offset_seconds {
                28800 => "Asia/Taipei".to_string(), // UTC+8
                25200 => "Asia/Jakarta".to_string(), // UTC+7
                32400 => "Asia/Tokyo".to_string(), // UTC+9
                0 => "UTC".to_string(),
                _ => "UTC".to_string(),
            }
        }
    }
}

#[tauri::command]
async fn fetch_prayer_times(location: String) -> Result<Vec<PrayerTime>, String> {
    let today = Local::now().format("%d-%m-%Y").to_string();
    let timezone_string = get_timezone_for_location(&location);

    let url = format!(
        "https://api.aladhan.com/v1/timingsByAddress/{}?address={}&method=3&shafaq=general&tune=5%2C3%2C5%2C7%2C9%2C-1%2C0%2C8%2C-6&timezonestring={}&calendarMethod=UAQ",
        today,
        urlencoding::encode(&location),
        urlencoding::encode(&timezone_string)
    );

    println!("Fetching prayer times for location: {} with timezone: {}", location, timezone_string);
    println!("URL: {}", url);

    let response = reqwest
        ::get(&url).await
        .map_err(|e| format!("Failed to fetch prayer times: {}", e))?;

    let prayer_response: PrayerTimesResponse = response
        .json().await
        .map_err(|e| format!("Failed to parse prayer times: {}", e))?;

    let mut prayer_times = Vec::new();
    let prayer_names = ["Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"];

    // Parse the timezone from the response or use the one we determined
    let tz: Tz = timezone_string
        .parse()
        .map_err(|_| format!("Invalid timezone: {}", timezone_string))?;

    for name in &prayer_names {
        if let Some(time_str) = prayer_response.data.timings.get(*name) {
            if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
                // Create datetime in the correct timezone
                let today_naive = Local::now().date_naive();
                let datetime = tz
                    .from_local_datetime(&today_naive.and_time(time))
                    .single()
                    .ok_or_else(|| format!("Invalid datetime for {}: {}", name, time_str))?
                    .with_timezone(&Local);

                prayer_times.push(PrayerTime {
                    name: name.to_string(),
                    time: time_str.clone(),
                    datetime,
                });
            }
        }
    }

    Ok(prayer_times)
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.lock().map_err(|e| format!("Failed to lock settings: {}", e))?;
    Ok(settings.clone())
}

#[tauri::command]
async fn save_settings(settings: AppSettings, state: State<'_, AppState>) -> Result<(), String> {
    println!("Settings saved: {:?}", settings);

    // Update the global state
    {
        let mut current_settings = state
            .lock()
            .map_err(|e| format!("Failed to lock settings: {}", e))?;
        *current_settings = settings.clone();
    }

    // Handle auto-start setting
    let auto_launch = AutoLaunch::new("Lets Pray", "", &[] as &[&str]);

    if settings.run_at_startup {
        auto_launch.enable().map_err(|e| format!("Failed to enable auto-start: {}", e))?;
    } else {
        auto_launch.disable().map_err(|e| format!("Failed to disable auto-start: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
async fn show_notification(_title: String, _body: String) -> Result<(), String> {
    // This will be handled by the notification plugin
    Ok(())
}

#[tauri::command]
async fn test_adhan_sound() -> Result<(), String> {
    // Emit event to frontend to play test sound
    Ok(())
}

#[tauri::command]
async fn get_system_info() -> Result<SystemInfo, String> {
    let timezone = get_windows_timezone().unwrap_or_else(|_| "UTC".to_string());
    let location = get_windows_location().unwrap_or_else(|_| "Unknown".to_string());

    Ok(SystemInfo {
        timezone,
        location,
    })
}

fn create_tray_icon(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create menu items
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Create menu
    let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

    // Create tray icon
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "settings" => {
                    println!("Settings menu item was clicked");
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                        let _ = window.center();
                    }
                }
                "quit" => {
                    println!("Quit menu item was clicked");
                    app.exit(0);
                }
                _ => {
                    println!("Menu item {:?} not handled", event.id);
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    println!("Left click pressed and released");
                    // Show and focus the main window when tray is clicked
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                        let _ = window.center();
                    }
                }
                _ => {
                    println!("Unhandled tray event {event:?}");
                }
            }
        })
        .build(app)?;

    Ok(())
}
async fn check_prayer_reminders(app_handle: AppHandle) {
    // Get settings from the global state
    let settings = {
        let state: State<AppState> = app_handle.state();
        let locked_state = state.lock().unwrap();
        locked_state.clone()
    };

    match fetch_prayer_times(settings.location).await {
        Ok(prayer_times) => {
            let now = Local::now();

            for prayer in prayer_times {
                if settings.enabled_prayers.contains(&prayer.name) {
                    let prayer_time = prayer.datetime;
                    let time_diff = prayer_time.signed_duration_since(now);

                    // Check if prayer time is now (within the current minute)
                    if
                        time_diff.num_minutes() == 0 &&
                        time_diff.num_seconds() >= 0 &&
                        time_diff.num_seconds() < 60
                    {
                        let title = format!("Prayer Time: {}", prayer.name);
                        let body = format!(
                            "It's time for {} prayer at {}",
                            prayer.name,
                            prayer.time
                        );

                        println!("Prayer time notification: {} at {}", prayer.name, prayer.time);

                        // Show the main window
                        if let Some(window) = app_handle.get_webview_window("main") {
                            println!("Showing main window for prayer time: {}", prayer.name);
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                            let _ = window.center();
                            // Bring window to front and make it always on top briefly
                            let _ = window.set_always_on_top(true);
                            // Reset always on top after a short delay
                            let window_clone = window.clone();
                            tauri::async_runtime::spawn(async move {
                                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                let _ = window_clone.set_always_on_top(false);
                            });
                        } else {
                            println!("Warning: Could not find main window to show");
                        }

                        // Send notification event to frontend
                        let _ = app_handle.emit(
                            "prayer-reminder",
                            serde_json::json!({
                             "title": title,
                             "body": body
                         })
                        );

                        // Play adhan sound if enabled
                        if settings.play_sound {
                            // Sound playing will be implemented in the frontend
                            let _ = app_handle.emit("play-adhan", ());
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch prayer times: {}", e);
        }
    }
}

fn setup_prayer_reminder_timer(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = interval(Duration::from_secs(60)); // Check every minute

        loop {
            interval.tick().await;
            check_prayer_reminders(app_handle.clone()).await;
        }
    });
}

// Window event handling is now done inline in the main function

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder
        ::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|app, event| {
            // Handle window events for main window
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    // Hide window instead of closing - minimize to tray
                    api.prevent_close();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                _ => {}
            }
        })
        .manage(AppState::new(AppSettings::default()))
        .setup(|app| {
            let app_handle = app.handle().clone();
            setup_prayer_reminder_timer(app_handle);

            // Create system tray icon
            if let Err(e) = create_tray_icon(app) {
                eprintln!("Failed to create tray icon: {}", e);
            }

            Ok(())
        })
        .invoke_handler(
            tauri::generate_handler![
                fetch_prayer_times,
                get_settings,
                save_settings,
                show_notification,
                test_adhan_sound,
                get_system_info
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
