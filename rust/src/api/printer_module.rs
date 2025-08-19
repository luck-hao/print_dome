// -------------------------- 1. 修复模块导入路径 --------------------------
// 假设 winapi_bindings.rs 在 src/api/ 目录下（若实际路径不同，需同步调整）
// 若 winapi_bindings.rs 与 lib.rs 同级，需改为 `use crate::winapi_bindings::*;`
use crate::api::winapi_bindings::*;

use flutter_rust_bridge::frb;
use std::{
    ffi::{c_void, CString, OsStr},
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
};
#[cfg(windows)]
use std::os::windows::prelude::OsStrExt;
use thiserror::Error;
use encoding_rs;

#[cfg(windows)]
use windows::{
    core::{PCWSTR, PSTR},
    Win32::Foundation::HANDLE,
};

// -------------------------- 错误定义 --------------------------
#[derive(Error, Debug, Clone, PartialEq)]
pub enum PrinterError {
    #[error("Windows API 调用失败: {func} (错误码: {code})")]
    WinApiError { func: &'static str, code: u32 },

    #[error("内存分配失败")]
    MemoryAllocFailed,

    #[error("打印机未找到: {0}")]
    PrinterNotFound(String),

    #[error("文件读取失败: {0}")]
    FileReadError(String),

    #[error("字符串编码失败")]
    EncodingError,

    #[error("无效参数: {0}")]
    InvalidParam(String),
    // 新增非 Windows 平台错误
    #[error("仅支持 Windows 系统")]
    UnsupportedOs,
}

// 实现错误到字符串的转换（供 Flutter 接收）
impl From<PrinterError> for String {
    fn from(e: PrinterError) -> Self {
        e.to_string()
    }
}

// -------------------------- 工具函数 --------------------------
/// 获取 Windows 最后错误码（仅 Windows 可用）
#[cfg(windows)]
fn get_last_error() -> u32 {
    unsafe { GetLastError() }
}

/// UTF-8 字符串转 UTF-16 宽字符（带 null 终止符，仅 Windows 可用）
#[cfg(windows)]
fn str_to_utf16(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(Some(0))
        .collect()
}

/// 字符串转 PCWSTR
#[cfg(windows)]
fn str_to_pcwstr(s: &str) -> windows::core::Result<PCWSTR> {
    let utf16 = str_to_utf16(s);
    Ok(PCWSTR(utf16.as_ptr()))
}

/// 字符串转 PSTR
#[cfg(windows)]
fn str_to_pstr(s: &str) -> windows::core::Result<PSTR> {
    let cstring = CString::new(s).map_err(|_| windows::core::Error::from_win32())?;
    Ok(PSTR(cstring.as_ptr() as *mut u8))
}

/// UTF-16 宽字符指针转 UTF-8 字符串（仅 Windows 可用）
#[cfg(windows)]
unsafe fn utf16_ptr_to_str(ptr: *const u16) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let mut len = 0;
    // 避免越界：限制最大长度（防止恶意指针导致无限循环）
    while len < 4096 && *ptr.add(len) != 0 {
        len += 1;
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16(slice).ok()
}

/// 分配 CoTask 内存（仅 Windows 可用）
#[cfg(windows)]
unsafe fn alloc_cotask_mem(size: usize) -> *mut c_void {
    if size > 1024 * 1024 * 100 {
        return null_mut();
    }
    CoTaskMemAlloc(size)
}

/// 释放 CoTask 内存（仅 Windows 可用）
#[cfg(windows)]
unsafe fn free_cotask_mem(ptr: *mut c_void) {
    if !ptr.is_null() {
        CoTaskMemFree(ptr);
    }
}

// -------------------------- 核心功能实现（仅 Windows 平台启用） --------------------------
/// 1. 获取本地打印机列表（对应 C# GetPrinterList）
#[flutter_rust_bridge::frb(sync)]
#[cfg(windows)]
pub fn get_printer_list() -> Result<Vec<String>, PrinterError> {
    unsafe {
        let mut cb_needed = 0;
        let mut c_returned = 0;

        // 第一步：获取所需缓冲区大小
        let success = unsafe {
            EnumPrintersW(
                PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS,
                PCWSTR::null(),
                2,
                None,
                0,
                &mut cb_needed,
                &mut c_returned,
            )
        };
        // 122 = ERROR_INSUFFICIENT_BUFFER（缓冲区不足，正常情况）
        if success == 0 && get_last_error() != 122 {
            return Err(PrinterError::WinApiError {
                func: "EnumPrintersW (1st call)",
                code: get_last_error(),
            });
        }

        // 检查缓冲区大小合理性（避免异常值）
        if cb_needed > 1024 * 1024 * 10 { // 限制最大 10MB
            return Err(PrinterError::MemoryAllocFailed);
        }

        // 分配缓冲区
        let mut buf = vec![0u8; cb_needed as usize];
        let buf_ptr = buf.as_mut_ptr();

        // 第二步：获取打印机信息
        let success = unsafe {
            EnumPrintersW(
                PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS,
                PCWSTR::null(),
                2,
                Some(buf_ptr),
                cb_needed,
                &mut cb_needed,
                &mut c_returned,
            )
        };
        if success == 0 {
            return Err(PrinterError::WinApiError {
                func: "EnumPrintersW (2nd call)",
                code: get_last_error(),
            });
        }

        // 解析 PRINTER_INFO_2W 数组
        let mut printers = Vec::new();
        let info_size = size_of::<PRINTER_INFO_2W>();
        // 检查结构体大小合理性（避免内存越界）
        if info_size == 0 {
            return Err(PrinterError::InvalidParam("PRINTER_INFO_2W 结构体大小为 0".to_string()));
        }

        for i in 0..c_returned {
            let info_ptr = (buf_ptr as usize + i as usize * info_size) as *mut PRINTER_INFO_2W;
            // 检查指针有效性
            if info_ptr.is_null() {
                continue;
            }
            let info = &*info_ptr;
            if let Some(name) = utf16_ptr_to_str(info.pPrinterName) {
                printers.push(name);
            }
        }

        Ok(printers)
    }
}

/// 2. 获取默认打印机名称（对应 C# GetDeaultPrinterName）
#[flutter_rust_bridge::frb(sync)]
#[cfg(windows)]
pub fn get_default_printer() -> Result<Option<String>, PrinterError> {
    unsafe {
        let mut buf_size = 0;
        // 第一步：获取所需缓冲区大小
        let success = GetDefaultPrinterW(null_mut(), &mut buf_size);
        if success == 0 && get_last_error() != 122 {
            return Err(PrinterError::WinApiError {
                func: "GetDefaultPrinterW (1st call)",
                code: get_last_error(),
            });
        }

        // 检查缓冲区大小合理性
        if buf_size > 4096 { // 打印机名称最大长度限制
            return Err(PrinterError::InvalidParam("默认打印机名称过长".to_string()));
        }

        // 分配缓冲区（宽字符数 + null 终止符）
        let mut buf = vec![0u16; buf_size as usize];
        let success = GetDefaultPrinterW(buf.as_mut_ptr(), &mut buf_size);
        if success == 0 {
            return Err(PrinterError::WinApiError {
                func: "GetDefaultPrinterW (2nd call)",
                code: get_last_error(),
            });
        }

        // 解析字符串（去掉 null 终止符，处理空缓冲区）
        if buf_size == 0 || buf.is_empty() {
            return Ok(None);
        }
        let name = String::from_utf16(&buf[0..(buf_size - 1) as usize]).map_err(|_| {
            PrinterError::EncodingError
        })?;
        Ok(Some(name))
    }
}

/// 3. 设置默认打印机（对应 C# SetPrinterToDefault）
#[flutter_rust_bridge::frb(sync)]
#[cfg(windows)]
pub fn set_default_printer(printer_name: &str) -> Result<(), PrinterError> {
    // 检查参数有效性
    if printer_name.is_empty() {
        return Err(PrinterError::InvalidParam("打印机名称不能为空".to_string()));
    }

    unsafe {
        let utf16_name = str_to_utf16(printer_name);
        let success = SetDefaultPrinterW(utf16_name.as_ptr());
        if success == 0 {
            return Err(PrinterError::WinApiError {
                func: "SetDefaultPrinterW",
                code: get_last_error(),
            });
        }
        Ok(())
    }
}

/// 4. 获取打印机状态（对应 C# GetPrinterStatus）
#[flutter_rust_bridge::frb(sync)]
#[cfg(windows)]
pub fn get_printer_status(printer_name: &str) -> Result<String, PrinterError> {
    // 检查参数有效性
    if printer_name.is_empty() {
        return Err(PrinterError::InvalidParam("打印机名称不能为空".to_string()));
    }

    unsafe {
        let utf16_name = str_to_utf16(printer_name);
        let mut h_printer: HANDLE = null_mut();
        let mut defaults: PRINTER_DEFAULTSW = zeroed();
        defaults.DesiredAccess = PRINTER_ACCESS_USE;

        // 打开打印机
        let success = OpenPrinterW(utf16_name.as_ptr(), &mut h_printer, &defaults);
        if success == 0 || h_printer.is_null() {
            return Err(PrinterError::WinApiError {
                func: "OpenPrinterW",
                code: get_last_error(),
            });
        }

        // 确保打印机句柄最终关闭（避免资源泄漏）
        let result = (|| {
            // 第一步：获取 PRINTER_INFO_2W 所需缓冲区大小
            let mut cb_needed = 0;
            let success = GetPrinterW(h_printer, 2, null_mut(), 0, &mut cb_needed);
            if success == 0 && get_last_error() != 122 {
                return Err(PrinterError::WinApiError {
                    func: "GetPrinterW (1st call)",
                    code: get_last_error(),
                });
            }

            // 检查缓冲区大小合理性
            if cb_needed > 1024 * 1024 * 5 { // 限制最大 5MB
                return Err(PrinterError::MemoryAllocFailed);
            }

            // 分配缓冲区并获取打印机信息
            let mut buf = vec![0u8; cb_needed as usize];
            let success = GetPrinterW(h_printer, 2, buf.as_mut_ptr(), cb_needed, &mut cb_needed);
            if success == 0 {
                return Err(PrinterError::WinApiError {
                    func: "GetPrinterW (2nd call)",
                    code: get_last_error(),
                });
            }

            // 解析状态
            let info_ptr = buf.as_ptr() as *const PRINTER_INFO_2W;
            if info_ptr.is_null() {
                return Err(PrinterError::InvalidParam("PRINTER_INFO_2W 指针为空".to_string()));
            }
            let status = (*info_ptr).Status;
            let status_str = match status {
                0 => "准备就绪（Ready）",
                0x00000200 => "忙(Busy）",
                0x00400000 => "门被打开（Printer Door Open）",
                0x00000002 => "错误(Printer Error）",
                0x00008000 => "正在初始化(Initializing）",
                0x00000100 => "正在输入或输出（I/O Active）",
                0x00000020 => "手工送纸（Manual Feed）",
                0x00040000 => "无墨粉（No Toner）",
                0x00001000 => "不可用（Not Available）",
                0x00000080 => "脱机（Off Line）",
                0x00200000 => "内存溢出（Out of Memory）",
                0x00000800 => "输出口已满（Output Bin Full）",
                0x00080000 => "当前页无法打印（Page Punt）",
                0x00000008 => "塞纸（Paper Jam）",
                0x00000010 => "打印纸用完（Paper Out）",
                0x00000040 => "纸张问题（Page Problem）",
                0x00000001 => "暂停（Paused）",
                0x00000004 => "正在删除（Pending Deletion）",
                0x00000400 => "正在打印（Printing）",
                0x00004000 => "正在处理（Processing）",
                0x00020000 => "墨粉不足（Toner Low）",
                0x00100000 => "需要用户干预（User Intervention）",
                0x20000000 => "等待（Waiting）",
                0x00010000 => "正在准备（Warming Up）",
                _ => "未知状态（Unknown Status）",
            };
            Ok(status_str.to_string())
        })();

        // 关闭打印机句柄（无论结果如何都要关闭）
        ClosePrinter(h_printer);
        result
    }
}

/// 5. 添加自定义纸张（对应 C# AddCustomPaperSize）
#[flutter_rust_bridge::frb(sync)]
#[cfg(windows)]
pub fn add_custom_paper(
    printer_name: &str,
    paper_name: &str,
    width_mm: f32,
    height_mm: f32,
) -> Result<(), PrinterError> {
    // 检查参数有效性
    if printer_name.is_empty() || paper_name.is_empty() {
        return Err(PrinterError::InvalidParam("打印机名称/纸张名称不能为空".to_string()));
    }
    if width_mm <= 0.0 || height_mm <= 0.0 || width_mm > 1000.0 || height_mm > 2000.0 {
        return Err(PrinterError::InvalidParam("纸张尺寸需在 (0, 1000]mm × (0, 2000]mm 范围内".to_string()));
    }

    unsafe {
        let utf16_printer = str_to_utf16(printer_name);
        let utf16_paper = str_to_utf16(paper_name);
        let mut h_printer: HANDLE = null_mut();
        let mut defaults: PRINTER_DEFAULTSW = zeroed();
        defaults.DesiredAccess = PRINTER_ACCESS_ADMINISTER | PRINTER_ACCESS_USE;

        // 打开打印机
        let success = OpenPrinterW(utf16_printer.as_ptr(), &mut h_printer, &defaults);
        if success == 0 || h_printer.is_null() {
            return Err(PrinterError::WinApiError {
                func: "OpenPrinterW",
                code: get_last_error(),
            });
        }

        // 确保资源最终释放（避免泄漏）
        let result = (|| {
            // 清理：先删除已存在的纸张（忽略删除失败，可能是纸张不存在）
            DeleteFormW(h_printer, utf16_paper.as_ptr());

            // 构建 FORM_INFO_1W（千毫米 = mm × 1000）
            let mut form_info: FORM_INFO_1W = zeroed();
            form_info.Flags = 0; // 0 = 系统定义纸张（非用户自定义）
            form_info.pName = utf16_paper.as_ptr();
            form_info.Size.cx = (width_mm * 1000.0) as INT;
            form_info.Size.cy = (height_mm * 1000.0) as INT;
            form_info.ImageableArea = RECTW {
                left: 0,
                top: 0,
                right: form_info.Size.cx,
                bottom: form_info.Size.cy,
            };

            // 添加自定义纸张
            let success = AddFormW(h_printer, 1, &mut form_info);
            if success == 0 {
                return Err(PrinterError::WinApiError {
                    func: "AddFormW",
                    code: get_last_error(),
                });
            }

            // -------------------------- 更新打印机设备模式 --------------------------
            // 1. 获取 DEVMODE 大小
            let devmode_size = DocumentPropertiesW(
                null_mut(),
                h_printer,
                utf16_printer.as_ptr(),
                null_mut(),
                null(),
                0,
            );
            if devmode_size < 0 {
                return Err(PrinterError::WinApiError {
                    func: "DocumentPropertiesW (get size)",
                    code: get_last_error(),
                });
            }

            // 2. 分配 DEVMODE 内存（+100 字节预留扩展空间）
            let devmode_ptr = alloc_cotask_mem(devmode_size as usize + 100) as *mut DEVMODEW;
            if devmode_ptr.is_null() {
                return Err(PrinterError::MemoryAllocFailed);
            }
            // 确保 DEVMODE 内存最终释放
            let devmode_result = (|| {
                // 3. 获取当前 DEVMODE
                let success = DocumentPropertiesW(
                    null_mut(),
                    h_printer,
                    utf16_printer.as_ptr(),
                    devmode_ptr,
                    null(),
                    DM_OUT_BUFFER,
                );
                if success < 0 {
                    return Err(PrinterError::WinApiError {
                        func: "DocumentPropertiesW (get devmode)",
                        code: get_last_error(),
                    });
                }

                // 4. 设置 DEVMODE（指定纸张名称）
                let mut devmode = *devmode_ptr;
                devmode.dmFields = 0x10000; // DM_FORMNAME = 0x10000（启用纸张名称设置）
                // 复制纸张名称到 dmFormName（32个宽字符限制，确保不越界）
                let paper_name_utf16 = str_to_utf16(paper_name);
                for (i, &c) in paper_name_utf16.iter().take(31).enumerate() {
                    devmode.dmFormName[i] = c;
                }
                devmode.dmFormName[31] = 0; // 确保 null 终止
                *devmode_ptr = devmode;

                // 5. 更新 DEVMODE 到打印机
                let success = DocumentPropertiesW(
                    null_mut(),
                    h_printer,
                    utf16_printer.as_ptr(),
                    devmode_ptr,
                    devmode_ptr,
                    DM_IN_BUFFER | DM_OUT_BUFFER,
                );
                if success < 0 {
                    return Err(PrinterError::WinApiError {
                        func: "DocumentPropertiesW (set devmode)",
                        code: get_last_error(),
                    });
                }

                // 6. 通过 PRINTER_INFO_9 更新打印机配置
                let mut cb_needed = 0;
                GetPrinterW(h_printer, 9, null_mut(), 0, &mut cb_needed);
                if cb_needed == 0 {
                    return Err(PrinterError::WinApiError {
                        func: "GetPrinterW (get PRINTER_INFO_9 size)",
                        code: get_last_error(),
                    });
                }

                let info_ptr = alloc_cotask_mem(cb_needed as usize) as *mut PRINTER_INFO_9W;
                if info_ptr.is_null() {
                    return Err(PrinterError::MemoryAllocFailed);
                }
                // 确保 PRINTER_INFO_9 内存最终释放
                let info_result = (|| {
                    let success = GetPrinterW(h_printer, 9, info_ptr as *mut u8, cb_needed, &mut cb_needed);
                    if success == 0 {
                        return Err(PrinterError::WinApiError {
                            func: "GetPrinterW (PRINTER_INFO_9)",
                            code: get_last_error(),
                        });
                    }

                    // 更新 DEVMODE 指针到 PRINTER_INFO_9
                    (*info_ptr).pDevMode = devmode_ptr;
                    let success = SetPrinterW(h_printer, 9, info_ptr as *mut u8, 0);
                    if success == 0 {
                        return Err(PrinterError::WinApiError {
                            func: "SetPrinterW",
                            code: get_last_error(),
                        });
                    }

                    // 7. 广播设置变更（通知系统更新打印机配置）
                    let mut result = 0;
                    SendMessageTimeoutW(
                        HWND_BROADCAST,
                        WM_SETTINGCHANGE,
                        null_mut() as WPARAM,
                        null_mut() as LPARAM,
                        SMTO_NORMAL,
                        1000, // 1秒超时
                        &mut result,
                    );

                    Ok(())
                })();

                // 释放 PRINTER_INFO_9 内存
                free_cotask_mem(info_ptr as *mut c_void);
                info_result
            })();

            // 释放 DEVMODE 内存（无论结果如何）
            free_cotask_mem(devmode_ptr as *mut c_void);
            devmode_result
        })();

        // 关闭打印机句柄（无论结果如何）
        ClosePrinter(h_printer);
        result
    }
}

/// 6. 删除自定义纸张（对应 C# DeleteCustomPaperSize）
#[flutter_rust_bridge::frb(sync)]
#[cfg(windows)]
pub fn delete_custom_paper(printer_name: &str, paper_name: &str) -> Result<(), PrinterError> {
    // 检查参数有效性
    if printer_name.is_empty() || paper_name.is_empty() {
        return Err(PrinterError::InvalidParam("打印机名称/纸张名称不能为空".to_string()));
    }

    unsafe {
        let utf16_printer = str_to_utf16(printer_name);
        let utf16_paper = str_to_utf16(paper_name);
        let mut h_printer: HANDLE = null_mut();
        let mut defaults: PRINTER_DEFAULTSW = zeroed();
        defaults.DesiredAccess = PRINTER_ACCESS_ADMINISTER | PRINTER_ACCESS_USE;

        // 打开打印机
        let success = OpenPrinterW(utf16_printer.as_ptr(), &mut h_printer, &defaults);
        if success == 0 || h_printer.is_null() {
            return Err(PrinterError::WinApiError {
                func: "OpenPrinterW",
                code: get_last_error(),
            });
        }

        // 确保句柄关闭
        let result = (|| {
            // 删除纸张
            let success = DeleteFormW(h_printer, utf16_paper.as_ptr());
            if success == 0 {
                return Err(PrinterError::WinApiError {
                    func: "DeleteFormW",
                    code: get_last_error(),
                });
            }
            Ok(())
        })();

        ClosePrinter(h_printer);
        result
    }
}

/// 7. 发送字节到打印机（对应 C# SendBytesToPrinter）
#[flutter_rust_bridge::frb(sync)]
#[cfg(windows)]
pub fn send_bytes_to_printer(printer_name: &str, bytes: &[u8]) -> Result<(), PrinterError> {
    // 检查参数有效性
    if printer_name.is_empty() {
        return Err(PrinterError::InvalidParam("打印机名称不能为空".to_string()));
    }
    if bytes.is_empty() {
        return Err(PrinterError::InvalidParam("打印字节数据不能为空".to_string()));
    }

    unsafe {
        let utf16_printer = str_to_utf16(printer_name);
        let mut h_printer: HANDLE = null_mut();

        // 打开打印机（无默认配置）
        let success = OpenPrinterW(utf16_printer.as_ptr(), &mut h_printer, null());
        if success == 0 || h_printer.is_null() {
            return Err(PrinterError::WinApiError {
                func: "OpenPrinterW",
                code: get_last_error(),
            });
        }

        // 确保资源释放（打印流程需严格释放）
        let result = (|| {
            // 构建 DOC_INFO_1W（Unicode 编码）
            let doc_name = str_to_utf16("Flutter-Rust RAW Document");
            let data_type = str_to_utf16("RAW");
            let doc_info = DOC_INFO_1W {
                pDocName: PCWSTR(doc_name.as_ptr()),
                pOutputFile: PCWSTR::null(), // NULL = 打印到打印机（非文件）
                pDataType: PCWSTR(data_type.as_ptr()),
            };

            // 开始打印文档（需检查返回值）
            let success = StartDocPrinterW(h_printer, 1, &doc_info);
            if success == 0 {
                return Err(PrinterError::WinApiError {
                    func: "StartDocPrinterW",
                    code: get_last_error(),
                });
            }

            // 确保打印文档结束（即使中间失败）
            let doc_result = (|| {
                // 开始打印页
                let success = StartPagePrinter(h_printer);
                if success == 0 {
                    return Err(PrinterError::WinApiError {
                        func: "StartPagePrinter",
                        code: get_last_error(),
                    });
                }

                // 确保打印页结束
                let page_result = (|| {
                    // 写入打印数据
                    let mut written = 0;
                    let success = WritePrinter(
                        h_printer,
                        bytes.as_ptr(),
                        bytes.len() as DWORD,
                        &mut written,
                    );
                    if success == 0 || written != bytes.len() as DWORD {
                        return Err(PrinterError::WinApiError {
                            func: "WritePrinter",
                            code: get_last_error(),
                        });
                    }
                    Ok(())
                })();

                // 结束打印页（无论写入结果如何）
                EndPagePrinter(h_printer);
                page_result
            })();

            // 结束打印文档（无论页结果如何）
            EndDocPrinter(h_printer);
            doc_result
        })();

        // 关闭打印机句柄（无论结果如何）
        ClosePrinter(h_printer);
        result
    }
}

/// 8. 发送文件到打印机（对应 C# SendFileToPrinter）
#[flutter_rust_bridge::frb(sync)]
#[cfg(windows)]
pub fn send_file_to_printer(printer_name: &str, file_path: &str) -> Result<(), PrinterError> {
    // 检查参数有效性
    if file_path.is_empty() {
        return Err(PrinterError::InvalidParam("文件路径不能为空".to_string()));
    }

    // 读取文件二进制内容（处理文件不存在/权限问题）
    let bytes = std::fs::read(file_path).map_err(|e| {
        PrinterError::FileReadError(format!("路径: {}，错误: {}", file_path, e))
    })?;
    send_bytes_to_printer(printer_name, &bytes)
}

/// 9. 发送字符串到打印机（对应 C# SendStringToPrinter）
#[flutter_rust_bridge::frb(sync)]
#[cfg(windows)]
pub fn send_string_to_printer(printer_name: &str, s: &str) -> Result<(), PrinterError> {
    // 检查参数有效性
    if s.is_empty() {
        return Err(PrinterError::InvalidParam("打印字符串不能为空".to_string()));
    }

    // 转 ANSI 编码（对应 C# StringToCoTaskMemAnsi，使用 Windows-1252 编码）
    let (ansi_bytes, _, had_errors) = encoding_rs::WINDOWS_1252.encode(s);
    if had_errors {
        return Err(PrinterError::EncodingError);
    }
    send_bytes_to_printer(printer_name, &ansi_bytes)
}

/// 10. 检查打印机是否存在（对应 C# PrinterInList）
#[flutter_rust_bridge::frb(sync)]
#[cfg(windows)]
pub fn is_printer_exist(printer_name: &str) -> Result<bool, PrinterError> {
    // 检查参数有效性
    if printer_name.is_empty() {
        return Err(PrinterError::InvalidParam("打印机名称不能为空".to_string()));
    }

    let printers = get_printer_list()?;
    Ok(printers.contains(&printer_name.to_string()))
}

// -------------------------- 非 Windows 平台占位函数（避免编译错误） --------------------------
/// 非 Windows 平台：获取打印机列表（返回不支持错误）
#[flutter_rust_bridge::frb(sync)]
#[cfg(not(windows))]
pub fn get_printer_list() -> Result<Vec<String>, PrinterError> {
    Err(PrinterError::UnsupportedOs)
}

/// 非 Windows 平台：获取默认打印机（返回不支持错误）
#[flutter_rust_bridge::frb(sync)]
#[cfg(not(windows))]
pub fn get_default_printer() -> Result<Option<String>, PrinterError> {
    Err(PrinterError::UnsupportedOs)
}

/// 非 Windows 平台：设置默认打印机（返回不支持错误）
#[flutter_rust_bridge::frb(sync)]
#[cfg(not(windows))]
pub fn set_default_printer(_printer_name: &str) -> Result<(), PrinterError> {
    Err(PrinterError::UnsupportedOs)
}

/// 非 Windows 平台：获取打印机状态（返回不支持错误）
#[flutter_rust_bridge::frb(sync)]
#[cfg(not(windows))]
pub fn get_printer_status(_printer_name: &str) -> Result<String, PrinterError> {
    Err(PrinterError::UnsupportedOs)
}

/// 非 Windows 平台：添加自定义纸张（返回不支持错误）
#[flutter_rust_bridge::frb(sync)]
#[cfg(not(windows))]
pub fn add_custom_paper(
    _printer_name: &str,
    _paper_name: &str,
    _width_mm: f32,
    _height_mm: f32,
) -> Result<(), PrinterError> {
    Err(PrinterError::UnsupportedOs)
}

/// 非 Windows 平台：删除自定义纸张（返回不支持错误）
#[flutter_rust_bridge::frb(sync)]
#[cfg(not(windows))]
pub fn delete_custom_paper(_printer_name: &str, _paper_name: &str) -> Result<(), PrinterError> {
    Err(PrinterError::UnsupportedOs)
}

/// 非 Windows 平台：发送字节到打印机（返回不支持错误）
#[flutter_rust_bridge::frb(sync)]
#[cfg(not(windows))]
pub fn send_bytes_to_printer(_printer_name: &str, _bytes: &[u8]) -> Result<(), PrinterError> {
    Err(PrinterError::UnsupportedOs)
}

/// 非 Windows 平台：发送文件到打印机（返回不支持错误）
#[flutter_rust_bridge::frb(sync)]
#[cfg(not(windows))]
pub fn send_file_to_printer(_printer_name: &str, _file_path: &str) -> Result<(), PrinterError> {
    Err(PrinterError::UnsupportedOs)
}

/// 非 Windows 平台：发送字符串到打印机（返回不支持错误）
#[flutter_rust_bridge::frb(sync)]
#[cfg(not(windows))]
pub fn send_string_to_printer(_printer_name: &str, _s: &str) -> Result<(), PrinterError> {
    Err(PrinterError::UnsupportedOs)
}

/// 非 Windows 平台：检查打印机是否存在（返回不支持错误）
#[flutter_rust_bridge::frb(sync)]
#[cfg(not(windows))]
pub fn is_printer_exist(_printer_name: &str) -> Result<bool, PrinterError> {
    Err(PrinterError::UnsupportedOs)
}

// -------------------------- 桥接初始化（必须启用，否则桥接函数无法调用） --------------------------
// flutter_rust_bridge::frb_init!();

// -------------------------- 测试模块（可选） --------------------------
// #[cfg(test)]
// #[cfg(windows)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_get_printer_list() {
//         let result = get_printer_list();
//         assert!(result.is_ok(), "获取打印机列表失败: {:?}", result);
//         let printers = result.unwrap();
//         println!("已检测到打印机: {:?}", printers);
//     }

//     #[test]
//     fn test_get_default_printer() {
//         let result = get_default_printer();
//         match result {
//             Ok(Some(name)) => println!("默认打印机: {}", name),
//             Ok(None) => println!("无默认打印机"),
//             Err(e) => assert!(false, "获取默认打印机失败: {:?}", e),
//         }
//     }
// }