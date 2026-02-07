use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Accessibility::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
use std::sync::OnceLock;
use std::sync::mpsc::Sender;


pub struct ActivityEvent {
    pub title: String,
    pub process_name: String,
}

static SENDER: OnceLock<Sender<ActivityEvent>> = OnceLock::new();

// Helper to get process name
unsafe fn get_process_name(hwnd: HWND) -> String {
    // We create a specific unsafe block for the API calls
    unsafe {
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, process_id);
        
        match handle {
            Ok(h) => {
                let mut buffer = [0u16; 260];
                let len = GetModuleBaseNameW(h, None, &mut buffer);
                let _ = CloseHandle(h); 
                
                if len > 0 {
                    String::from_utf16_lossy(&buffer[..len as usize])
                } else {
                    "Unknown".to_string()
                }
            }
            Err(_) => "System/Protected".to_string(),
        }
    }
}

// The Callback Function
unsafe extern "system" fn win_event_proc(
    _h_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_obj: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    // Wrap everything in catch_unwind to prevent hard crashes
    let _ = std::panic::catch_unwind(|| {
        unsafe {
            if event == EVENT_SYSTEM_FOREGROUND {
                // ... rest of the logic should be here ...
                let mut buffer = [0u16; 512];
                let len = GetWindowTextW(hwnd, &mut buffer);
                
                if len > 0 {
                    let title = String::from_utf16_lossy(&buffer[..len as usize]);
                    let process_name = get_process_name(hwnd);
                    
                    if let Some(tx) = SENDER.get() {
                        let _ = tx.send(ActivityEvent { title, process_name });
                    }
                }
            }
        }
    });
}

pub fn start_event_loop(tx: Sender<ActivityEvent>) {
    let _ = SENDER.set(tx);
    
    unsafe {
        let _hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }
    }
}