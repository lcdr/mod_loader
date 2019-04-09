#![feature(asm)]
#![feature(naked_functions)]

use std::ffi::{CStr, CString};
use std::fs;
use std::io::Result as Res;
use std::os::raw::c_char;

static mut FUNCS: [FARPROC; 5] = [0 as FARPROC; 5];
static mut HAS_CONSOLE: bool = false;

use winapi::{
	shared::{
		minwindef::{BOOL, DWORD, FALSE, FARPROC, HINSTANCE, LPVOID, TRUE},
		ntdef::HANDLE,
	},
	um::{
		libloaderapi::{GetProcAddress, LoadLibraryA},
		consoleapi::AllocConsole,
		processenv::SetStdHandle,
		winbase::STD_OUTPUT_HANDLE,
	}
};

#[no_mangle]
#[allow(unused_variables)]
pub extern "system" fn DllMain(
	dll_module: HINSTANCE,
	call_reason: DWORD,
	reserved: LPVOID)
	-> BOOL {
	const DLL_PROCESS_ATTACH: DWORD = 1;

	match call_reason {
		DLL_PROCESS_ATTACH => init(),
		_ => TRUE
	}
}

macro_rules! error {
	($($arg:tt)*) => {
		unsafe {
			if !HAS_CONSOLE {
				AllocConsole();
				SetStdHandle(STD_OUTPUT_HANDLE, 0 as HANDLE);
				HAS_CONSOLE = true;
			}
			eprintln!($($arg)*);
		}
	}
}

unsafe fn s(bytes: &[u8]) -> *const c_char {
	CStr::from_bytes_with_nul_unchecked(bytes).as_ptr()
}

fn init() -> BOOL {
	let dinput = unsafe { LoadLibraryA(s(b"C:\\Windows\\System32\\dinput8.dll\0")) };
	if dinput.is_null() {
		error!("modloader error: could not load real dinput8");
		return FALSE;
	}
	unsafe {
		FUNCS[0] = GetProcAddress(dinput, s(b"DirectInput8Create\0"));
		FUNCS[1] = GetProcAddress(dinput, s(b"DllCanUnloadNow\0"));
		FUNCS[2] = GetProcAddress(dinput, s(b"DllGetClassObject\0"));
		FUNCS[3] = GetProcAddress(dinput, s(b"DllRegisterServer\0"));
		FUNCS[4] = GetProcAddress(dinput, s(b"DllUnregisterServer\0"));
	}

	match load_mods() {
		Ok(()) => TRUE,
		Err(e) => {
			error!("modloader error: {}", e);
			FALSE
		}
	}
}

fn load_mods() -> Res<()> {
	for entry in fs::read_dir("mods")? {
		let entry = entry.unwrap();
		if entry.file_type()?.is_dir() {
			let mut path = entry.path();
			path.push("mod.dll");
			let string = path.as_os_str().to_str().unwrap();
			let cstring = CString::new(string)?;
			let m = unsafe { LoadLibraryA(cstring.as_c_str().as_ptr()) };
			if m.is_null() {
				error!("modloader error: could not load mod: {}", string);
			} else {
				println!("modloader: sucessfully loaded {}", string);
			}
		}
	}
	Ok(())
}

#[naked]#[no_mangle]pub extern "system" fn DirectInput8Create()  { unsafe { asm!("jmp *$0":: "r"(FUNCS[0])); }}
#[naked]#[no_mangle]pub extern "system" fn DllCanUnloadNow()     { unsafe { asm!("jmp *$0":: "r"(FUNCS[1])); }}
#[naked]#[no_mangle]pub extern "system" fn DllGetClassObject()   { unsafe { asm!("jmp *$0":: "r"(FUNCS[2])); }}
#[naked]#[no_mangle]pub extern "system" fn DllRegisterServer()   { unsafe { asm!("jmp *$0":: "r"(FUNCS[3])); }}
#[naked]#[no_mangle]pub extern "system" fn DllUnregisterServer() { unsafe { asm!("jmp *$0":: "r"(FUNCS[4])); }}
