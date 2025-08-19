use std::ffi::c_void;

// -------------------------- 导入 windows crate 类型（避免重复定义） --------------------------
use windows::{
    core::{PCWSTR, PSTR},
    Win32::{
        Foundation::{BOOL, HANDLE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Printing::*,
        System::{
            Com::{CoTaskMemAlloc, CoTaskMemFree},
            Memory::LocalFree,
        },
        UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_NORMAL, WM_SETTINGCHANGE},
    },
};

pub type DWORD = u32;
pub type WORD = u16;
pub type INT = i32;
pub type UINT = u32;

// -------------------------- 核心结构体绑定（保留#[repr(C)]确保C内存布局） --------------------------
/// 打印机默认配置
#[repr(C)]
pub struct PRINTER_DEFAULTSW {
    pub pDatatype: PCWSTR,
    pub pDevMode: *mut DEVMODEW,
    pub DesiredAccess: DWORD,
}

/// 设备模式
#[repr(C)]
pub struct DEVMODEW {
    pub dmDeviceName: [u16; 32],
    pub dmSpecVersion: WORD,
    pub dmDriverVersion: WORD,
    pub dmSize: WORD,
    pub dmDriverExtra: WORD,
    pub dmFields: DWORD,
    pub dmOrientation: INT,
    pub dmPaperSize: INT,
    pub dmPaperLength: INT,
    pub dmPaperWidth: INT,
    pub dmScale: INT,
    pub dmCopies: INT,
    pub dmDefaultSource: INT,
    pub dmPrintQuality: INT,
    pub dmColor: INT,
    pub dmDuplex: INT,
    pub dmYResolution: INT,
    pub dmTTOption: INT,
    pub dmCollate: INT,
    pub dmFormName: [u16; 32],
    pub dmLogPixels: WORD,
    pub dmBitsPerPel: DWORD,
    pub dmPelsWidth: DWORD,
    pub dmPelsHeight: DWORD,
    pub dmNup: DWORD,
    pub dmDisplayFrequency: DWORD,
    pub dmICMMethod: DWORD,
    pub dmICMIntent: DWORD,
    pub dmMediaType: DWORD,
    pub dmDitherType: DWORD,
    pub dmReserved1: DWORD,
    pub dmReserved2: DWORD,
}

/// 尺寸结构体
#[repr(C)]
pub struct SIZEW {
    pub cx: INT,
    pub cy: INT,
}

/// 矩形结构体
#[repr(C)]
pub struct RECTW {
    pub left: INT,
    pub top: INT,
    pub right: INT,
    pub bottom: INT,
}

/// 纸张信息
#[repr(C)]
pub struct FORM_INFO_1W {
    pub Flags: DWORD,
    pub pName: PCWSTR,
    pub Size: SIZEW,
    pub ImageableArea: RECTW,
}

/// 打印机详细信息
#[repr(C)]
pub struct PRINTER_INFO_2W {
    pub pServerName: PCWSTR,
    pub pPrinterName: PCWSTR,
    pub pShareName: PCWSTR,
    pub pPortName: PCWSTR,
    pub pDriverName: PCWSTR,
    pub pComment: PCWSTR,
    pub pLocation: PCWSTR,
    pub pDevMode: *mut DEVMODEW,
    pub pSepFile: PCWSTR,
    pub pPrintProcessor: PCWSTR,
    pub pDatatype: PCWSTR,
    pub pParameters: PCWSTR,
    pub pSecurityDescriptor: *mut c_void,
    pub Attributes: DWORD,
    pub Priority: DWORD,
    pub DefaultPriority: DWORD,
    pub StartTime: DWORD,
    pub UntilTime: DWORD,
    pub Status: DWORD,
    pub cJobs: DWORD,
    pub AveragePPM: DWORD,
}

/// 每用户打印机配置
#[repr(C)]
pub struct PRINTER_INFO_9W {
    pub pDevMode: *mut DEVMODEW,
}

/// 打印文档信息
#[repr(C)]
pub struct DOC_INFO_1W {
    pub pDocName: PCWSTR,
    pub pOutputFile: PCWSTR,
    pub pDataType: PCWSTR,
}

// -------------------------- 常量定义 --------------------------
pub const PRINTER_ACCESS_ADMINISTER: DWORD = 0x00000004;
pub const PRINTER_ACCESS_USE: DWORD = 0x00000008;
pub const DM_OUT_BUFFER: DWORD = 0x00000002;
pub const DM_IN_BUFFER: DWORD = 0x00000008;
pub const PRINTER_ENUM_LOCAL: DWORD = 0x00000002;
pub const PRINTER_ENUM_CONNECTIONS: DWORD = 0x00000004;

// -------------------------- 导出 windows crate 函数 --------------------------
pub use windows::Win32::Graphics::Printing::{
    AddFormW, ClosePrinter, DeleteFormW, DocumentPropertiesW, EndDocPrinter, EndPagePrinter,
    EnumPrintersW, GetDefaultPrinterW, GetPrinterW, OpenPrinterW, SetDefaultPrinterW,
    SetPrinterW, StartDocPrinterW, StartPagePrinter, WritePrinter,
};

pub use windows::{
    core::Error as WindowsError,
    Win32::{
        Foundation::GetLastError,
        UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_NORMAL, WM_SETTINGCHANGE},
    },
};