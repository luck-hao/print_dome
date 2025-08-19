use std::ffi::c_void;
use winapi::ctypes::c_char;

// -------------------------- 1. 统一导入winapi类型（避免重复定义） --------------------------
// 直接从winapi导入所有基础类型，不手动定义同名类型
use winapi::shared::minwindef::{BOOL, DWORD, INT, WORD, LRESULT, UINT, WPARAM, LPARAM};
use winapi::shared::windef::HWND;
use winapi::shared::basetsd::{UINT_PTR, ULONG_PTR};
use winapi::um::winnt::HANDLE;

// -------------------------- 2. 仅导出必要类型（避免重复导出） --------------------------
// 仅导出当前模块需要对外暴露的类型，不重复导出winapi已有的类型
pub use self::{
    PRINTER_DEFAULTSW, DEVMODEW, SIZEW, RECTW, FORM_INFO_1W,
    PRINTER_INFO_2W, PRINTER_INFO_9W, DOC_INFO_1A
};

// -------------------------- 3. 核心结构体绑定（保留#[repr(C)]确保C内存布局） --------------------------
/// 打印机默认配置（对应 C# structPrinterDefaults）
#[repr(C)]
pub struct PRINTER_DEFAULTSW {
    pub pDatatype: *const u16,
    pub pDevMode: *mut DEVMODEW,
    pub DesiredAccess: DWORD,
}

/// 设备模式（对应 C# structDevMode）
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
    pub dmFormName: [u16; 32], // 纸张名称（32个宽字符）
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

/// 尺寸结构体（对应 C# structSize）
#[repr(C)]
pub struct SIZEW {
    pub cx: INT, // 宽度（千毫米）
    pub cy: INT, // 高度（千毫米）
}

/// 矩形结构体（对应 C# structRect）
#[repr(C)]
pub struct RECTW {
    pub left: INT,
    pub top: INT,
    pub right: INT,
    pub bottom: INT,
}

/// 纸张信息（对应 C# FormInfo1）
#[repr(C)]
pub struct FORM_INFO_1W {
    pub Flags: DWORD,
    pub pName: *const u16, // 纸张名称
    pub Size: SIZEW,       // 纸张尺寸
    pub ImageableArea: RECTW, // 可打印区域
}

/// 打印机详细信息（对应 C# PRINTER_INFO_2）
#[repr(C)]
pub struct PRINTER_INFO_2W {
    pub pServerName: *const u16,
    pub pPrinterName: *const u16, // 打印机名称
    pub pShareName: *const u16,
    pub pPortName: *const u16,
    pub pDriverName: *const u16,
    pub pComment: *const u16,
    pub pLocation: *const u16,
    pub pDevMode: *mut DEVMODEW,
    pub pSepFile: *const u16,
    pub pPrintProcessor: *const u16,
    pub pDatatype: *const u16,
    pub pParameters: *const u16,
    pub pSecurityDescriptor: *mut c_void,
    pub Attributes: DWORD,
    pub Priority: DWORD,
    pub DefaultPriority: DWORD,
    pub StartTime: DWORD,
    pub UntilTime: DWORD,
    pub Status: DWORD, // 打印机状态
    pub cJobs: DWORD,
    pub AveragePPM: DWORD,
}

/// 每用户打印机配置（对应 C# PRINTER_INFO_9）
#[repr(C)]
pub struct PRINTER_INFO_9W {
    pub pDevMode: *mut DEVMODEW,
}

/// 打印文档信息（对应 C# DOCINFOA）
#[repr(C)]
pub struct DOC_INFO_1A {
    pub pDocName: *const c_char,  // 文档名称（ANSI）
    pub pOutputFile: *const c_char, // 输出文件（NULL 表示打印到打印机）
    pub pDataType: *const c_char,  // 数据类型（RAW）
}

// -------------------------- 4. Windows API 函数绑定（修复库名+删除重复函数） --------------------------
// 1. 打印机相关API（依赖 winspool.drv 库，库名必须带 .drv）
#[link(name = "winspool")]
extern "system" {
    // 打开打印机
    pub fn OpenPrinterW(
        pPrinterName: *const u16,
        phPrinter: *mut HANDLE,
        pDefault: *const PRINTER_DEFAULTSW,
    ) -> BOOL;

    // 关闭打印机
    pub fn ClosePrinter(hPrinter: HANDLE) -> BOOL;

    // 枚举打印机
    pub fn EnumPrintersW(
        Flags: DWORD,
        pName: *const u16,
        Level: DWORD,
        pPrinterEnum: *mut u8,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
        pcReturned: *mut DWORD,
    ) -> BOOL;

    // 获取打印机信息
    pub fn GetPrinterW(
        hPrinter: HANDLE,
        Level: DWORD,
        pPrinter: *mut u8,
        cbBuf: DWORD,
        pcbNeeded: *mut DWORD,
    ) -> BOOL;

    // 获取默认打印机
    pub fn GetDefaultPrinterW(pDefaultPrinter: *mut u16, pcchBuffer: *mut DWORD) -> BOOL;

    // 设置默认打印机
    pub fn SetDefaultPrinterW(pPrinterName: *const u16) -> BOOL;

    // 添加自定义纸张
    pub fn AddFormW(hPrinter: HANDLE, Level: DWORD, pForm: *mut FORM_INFO_1W) -> BOOL;

    // 删除自定义纸张
    pub fn DeleteFormW(hPrinter: HANDLE, pFormName: *const u16) -> BOOL;

    // 获取/设置设备模式
    pub fn DocumentPropertiesW(
        hWnd: HWND,
        hPrinter: HANDLE,
        pDeviceName: *const u16,
        pDevModeOutput: *mut DEVMODEW,
        pDevModeInput: *const DEVMODEW,
        fMode: DWORD,
    ) -> INT;

    // 设置打印机配置
    pub fn SetPrinterW(
        hPrinter: HANDLE,
        Level: DWORD,
        pPrinter: *mut u8,
        Command: DWORD,
    ) -> BOOL;

    // 开始打印文档
    pub fn StartDocPrinterA(
        hPrinter: HANDLE,
        Level: DWORD,
        pDocInfo: *const DOC_INFO_1A,
    ) -> BOOL;

    // 结束打印文档
    pub fn EndDocPrinter(hPrinter: HANDLE) -> BOOL;

    // 开始打印页
    pub fn StartPagePrinter(hPrinter: HANDLE) -> BOOL;

    // 结束打印页
    pub fn EndPagePrinter(hPrinter: HANDLE) -> BOOL;

    // 写入打印数据
    pub fn WritePrinter(
        hPrinter: HANDLE,
        pBuf: *const u8,
        cbBuf: DWORD,
        pcWritten: *mut DWORD,
    ) -> BOOL;
}

// 2. 窗口消息相关API（依赖 user32.dll 库，单独绑定避免与打印机库混淆）
#[link(name = "user32")]
extern "system" {
    // 发送系统广播（通知设置变更）- 从user32.dll导入，而非winspool
    pub fn SendMessageTimeoutW(
        hWnd: HWND,
        Msg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        fuFlags: UINT,
        uTimeout: UINT,
        lpdwResult: *mut DWORD,
    ) -> LRESULT;
}

// -------------------------- 5. 常量定义（复用winapi+补充必要常量） --------------------------
// 1. 复用winapi已有的常量（避免重复定义）
pub use winapi::um::winuser::{WM_SETTINGCHANGE, HWND_BROADCAST, SMTO_NORMAL};

// 2. 补充打印机相关的自定义常量（winapi中无定义的）
pub const PRINTER_ACCESS_ADMINISTER: DWORD = 0x00000004;
pub const PRINTER_ACCESS_USE: DWORD = 0x00000008;
pub const DM_OUT_BUFFER: DWORD = 0x00000002;
pub const DM_IN_BUFFER: DWORD = 0x00000008;
pub const PRINTER_ENUM_LOCAL: DWORD = 0x00000002;
pub const PRINTER_ENUM_CONNECTIONS: DWORD = 0x00000004;

// -------------------------- 6. 复用winapi工具函数（避免重复绑定） --------------------------
// 直接导出winapi的工具函数，不手动绑定
pub use winapi::um::errhandlingapi::GetLastError;
pub use winapi::um::combaseapi::{CoTaskMemAlloc, CoTaskMemFree};