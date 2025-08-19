import 'package:flutter/material.dart';
import './services/printer_service.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '打印机桥接示例',
      theme: ThemeData(primarySwatch: Colors.blue),
      home: const PrinterTestPage(),
    );
  }
}

class PrinterTestPage extends StatefulWidget {
  const PrinterTestPage({super.key});

  @override
  State<PrinterTestPage> createState() => _PrinterTestPageState();
}

class _PrinterTestPageState extends State<PrinterTestPage> {
  final PrinterService _printerService = PrinterService();
  List<String> _printers = [];
  String? _selectedPrinter;
  String? _defaultPrinter;
  String _status = "未查询";
  String _log = "";

  // 自定义纸张参数
  final TextEditingController _paperNameCtrl = TextEditingController(
    text: "MyCustomPaper",
  );
  final TextEditingController _widthCtrl = TextEditingController(text: "80");
  final TextEditingController _heightCtrl = TextEditingController(text: "100");

  // 打印字符串参数
  final TextEditingController _printStrCtrl = TextEditingController(
    text: "Hello Flutter-Rust Printer!",
  );

  @override
  void initState() {
    super.initState();
    _loadPrinters();
    _loadDefaultPrinter();
  }

  // 加载打印机列表
  Future<void> _loadPrinters() async {
    try {
      final printers = await _printerService.getPrinterList();
      setState(() {
        _printers = printers;
        _selectedPrinter = printers.isNotEmpty ? printers.first : null;
      });
      _addLog("加载打印机列表成功: ${printers.length} 台");
    } catch (e) {
      _addLog(e.toString());
    }
  }

  // 加载默认打印机
  Future<void> _loadDefaultPrinter() async {
    try {
      final defaultPrinter = await _printerService.getDefaultPrinter();
      setState(() => _defaultPrinter = defaultPrinter);
      _addLog("默认打印机: ${defaultPrinter ?? "无"}");
    } catch (e) {
      _addLog(e.toString());
    }
  }

  // 查询打印机状态
  Future<void> _queryStatus() async {
    if (_selectedPrinter == null) {
      _addLog("请选择打印机");
      return;
    }
    try {
      final status = await _printerService.getPrinterStatus(_selectedPrinter!);
      setState(() => _status = status);
      _addLog("打印机状态: $status");
    } catch (e) {
      _addLog(e.toString());
    }
  }

  // 添加自定义纸张
  Future<void> _addCustomPaper() async {
    if (_selectedPrinter == null) {
      _addLog("请选择打印机");
      return;
    }
    try {
      final paperName = _paperNameCtrl.text;
      final width = double.parse(_widthCtrl.text);
      final height = double.parse(_heightCtrl.text);
      await _printerService.addCustomPaper(
        printerName: _selectedPrinter!,
        paperName: paperName,
        widthMm: width,
        heightMm: height,
      );
      _addLog("添加自定义纸张成功: $paperName (${width}mm x ${height}mm)");
    } catch (e) {
      _addLog(e.toString());
    }
  }

  // 发送字符串打印
  Future<void> _sendStringPrint() async {
    if (_selectedPrinter == null) {
      _addLog("请选择打印机");
      return;
    }
    try {
      final content = _printStrCtrl.text;
      await _printerService.sendStringToPrinter(_selectedPrinter!, content);
      _addLog("发送字符串打印成功: $content");
    } catch (e) {
      _addLog(e.toString());
    }
  }

  // 添加日志
  void _addLog(String content) {
    setState(() {
      _log = "[${DateTime.now().toString().substring(11, 19)}] $content\n$_log";
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text("打印机桥接测试")),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: ListView(
          children: [
            // 打印机选择
            DropdownButtonFormField<String>(
              value: _selectedPrinter,
              items: _printers.map((printer) {
                return DropdownMenuItem(value: printer, child: Text(printer));
              }).toList(),
              onChanged: (value) => setState(() => _selectedPrinter = value),
              decoration: const InputDecoration(labelText: "选择打印机"),
            ),
            const SizedBox(height: 16),

            // 默认打印机显示
            Text("默认打印机: ${_defaultPrinter ?? "无"}"),
            const SizedBox(height: 8),

            // 打印机状态
            Text("当前状态: $_status"),
            ElevatedButton(onPressed: _queryStatus, child: const Text("查询状态")),
            const SizedBox(height: 16),

            // 自定义纸张配置
            const Text("=== 自定义纸张 ==="),
            TextField(
              controller: _paperNameCtrl,
              decoration: const InputDecoration(labelText: "纸张名称"),
            ),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _widthCtrl,
                    keyboardType: TextInputType.number,
                    decoration: const InputDecoration(labelText: "宽度(mm)"),
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: TextField(
                    controller: _heightCtrl,
                    keyboardType: TextInputType.number,
                    decoration: const InputDecoration(labelText: "高度(mm)"),
                  ),
                ),
              ],
            ),
            ElevatedButton(
              onPressed: _addCustomPaper,
              child: const Text("添加自定义纸张"),
            ),
            const SizedBox(height: 16),

            // 打印字符串
            const Text("=== 打印测试 ==="),
            TextField(
              controller: _printStrCtrl,
              decoration: const InputDecoration(labelText: "打印内容"),
            ),
            ElevatedButton(
              onPressed: _sendStringPrint,
              child: const Text("发送打印"),
            ),
            const SizedBox(height: 16),

            // 日志区域
            const Text("=== 操作日志 ==="),
            Container(
              padding: const EdgeInsets.all(8),
              height: 200,
              decoration: BoxDecoration(
                border: Border.all(color: Colors.grey),
                borderRadius: BorderRadius.circular(4),
              ),
              child: SingleChildScrollView(child: Text(_log)),
            ),
          ],
        ),
      ),
    );
  }
}
