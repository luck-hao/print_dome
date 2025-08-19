import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import '../src/rust/api/printer_module.dart' as api;

class PrinterService {
  // 单例
  static final PrinterService _instance = PrinterService._();
  factory PrinterService() => _instance;
  PrinterService._();

  // 1. 获取打印机列表
  Future<List<String>> getPrinterList() async {
    try {
      final result = await api.getPrinterList();
      return result ?? [];
    } catch (e) {
      throw _formatError(e);
    }
  }

  // 2. 获取默认打印机
  Future<String?> getDefaultPrinter() async {
    try {
      final result = await api.getDefaultPrinter();
      return result;
    } catch (e) {
      throw _formatError(e);
    }
  }

  // 3. 设置默认打印机
  Future<void> setDefaultPrinter(String printerName) async {
    try {
      api.setDefaultPrinter(printerName: printerName);
    } catch (e) {
      throw _formatError(e);
    }
  }

  // 4. 获取打印机状态
  Future<String> getPrinterStatus(String printerName) async {
    try {
      final result = await api.getPrinterStatus(printerName: printerName);
      return result ?? "未知状态";
    } catch (e) {
      throw _formatError(e);
    }
  }

  // 5. 添加自定义纸张（mm 为单位）
  Future<void> addCustomPaper({
    required String printerName,
    required String paperName,
    required double widthMm,
    required double heightMm,
  }) async {
    try {
      api.addCustomPaper(
        printerName: printerName,
        paperName: paperName,
        widthMm: widthMm,
        heightMm: heightMm,
      );
    } catch (e) {
      throw _formatError(e);
    }
  }

  // 6. 删除自定义纸张
  Future<void> deleteCustomPaper({
    required String printerName,
    required String paperName,
  }) async {
    try {
      api.deleteCustomPaper(printerName: printerName, paperName: paperName);
    } catch (e) {
      throw _formatError(e);
    }
  }

  // 7. 发送字节到打印机
  Future<void> sendBytesToPrinter(String printerName, List<int> bytes) async {
    try {
      api.sendBytesToPrinter(printerName: printerName, bytes: bytes);
    } catch (e) {
      throw _formatError(e);
    }
  }

  // 8. 发送文件到打印机
  Future<void> sendFileToPrinter(String printerName, String filePath) async {
    try {
      api.sendFileToPrinter(printerName: printerName, filePath: filePath);
    } catch (e) {
      throw _formatError(e);
    }
  }

  // 9. 发送字符串到打印机
  Future<void> sendStringToPrinter(String printerName, String content) async {
    try {
      api.sendStringToPrinter(printerName: printerName, s: content);
    } catch (e) {
      throw _formatError(e);
    }
  }

  // 10. 检查打印机是否存在
  Future<bool> isPrinterExist(String printerName) async {
    try {
      final result = await api.isPrinterExist(printerName: printerName);
      return result ?? false;
    } catch (e) {
      throw _formatError(e);
    }
  }

  // 错误格式化
  String _formatError(dynamic e) {
    if (e is FrbException) {
      return "打印机错误: ${e.message}";
    } else {
      return "未知错误: ${e.toString()}";
    }
  }
}

extension on FrbException {
  get message => null;
}
