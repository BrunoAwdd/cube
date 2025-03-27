import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;
import 'package:qr_code_scanner/qr_code_scanner.dart';
import 'package:shared_preferences/shared_preferences.dart';

class PairingPage extends StatefulWidget {
  const PairingPage({super.key});

  @override
  State<PairingPage> createState() => _PairingPageState();
}

class _PairingPageState extends State<PairingPage> {
  final GlobalKey qrKey = GlobalKey(debugLabel: 'QR');
  final TextEditingController manualController = TextEditingController();
  bool _scanned = false;

  Future<void> _authenticate(String ip, String code, String username) async {
    try {
      final res = await http.post(
        Uri.parse("http://$ip:8080/auth"),
        headers: {"Content-Type": "application/json"},
        body: jsonEncode({"code": code, "username": username}),
      );

      if (res.statusCode == 200) {
        final token = jsonDecode(res.body)['token'];
        final prefs = await SharedPreferences.getInstance();
        await prefs.setString("ip", ip);
        await prefs.setString("token", token);
        await prefs.setString("username", username);

        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text("✅ Pareado com sucesso!")),
        );

        Navigator.pushReplacementNamed(context, '/gallery');
      } else {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text("❌ Código inválido")),
        );
        setState(() => _scanned = false);
      }
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text("❌ Erro na conexão: $e")),
      );
      setState(() => _scanned = false);
    }
  }

  Future<void> _processLink(String link) async {
    try {
      final uri = Uri.parse(link);
      final code = uri.queryParameters['code'];
      final ip = uri.host;

      if (code == null || ip.isEmpty) throw "Link inválido";

      const username = "bruno";
      await _authenticate(ip, code, username);
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text("❌ Erro: $e")),
      );
      setState(() => _scanned = false);
    }
  }

  void _onQRViewCreated(QRViewController controller) {
    controller.scannedDataStream.listen((scanData) async {
      if (_scanned) return;
      _scanned = true;

      await _processLink(scanData.code ?? '');
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Pareamento com o Servidor')),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: manualController,
                    decoration: const InputDecoration(
                      labelText: 'Colar link de pareamento',
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                ElevatedButton(
                  onPressed: () => _processLink(manualController.text),
                  child: const Text('Conectar'),
                ),
              ],
            ),
          ),
          Expanded(
            child: QRView(
              key: qrKey,
              onQRViewCreated: _onQRViewCreated,
            ),
          ),
        ],
      ),
    );
  }
}
