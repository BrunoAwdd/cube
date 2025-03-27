import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:http/http.dart' as http;
import 'package:provider/provider.dart';
import '../services/ws_service.dart';

class SplashPage extends StatefulWidget {
  const SplashPage({super.key});

  @override
  State<SplashPage> createState() => _SplashPageState();
}

class _SplashPageState extends State<SplashPage> {
  @override
  void initState() {
    super.initState();
    _checkPairing();
  }

  Future<void> _checkPairing() async {
    final prefs = await SharedPreferences.getInstance();
    final username = prefs.getString("username");

    if (username == null || username.isEmpty) {
      Navigator.pushReplacementNamed(context, '/login');
      return;
    }

    final ip = prefs.getString("ip");
    final token = prefs.getString("token");

    if (ip != null && token != null) {
      try {
        final res = await http.get(Uri.parse("http://$ip:8080/poll_token?token=$token"));
        if (res.statusCode == 200) {
          // ✅ Conecta ao WebSocket ANTES de ir pra gallery
          final ws = Provider.of<WebSocketService>(context, listen: false);
          ws.connect(ip);

          Navigator.pushReplacementNamed(context, '/gallery');
          return;
        }
      } catch (_) {}
    }

    // Falhou ou sem dados → voltar para pareamento
    Navigator.pushReplacementNamed(context, '/pair');
  }

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(child: CircularProgressIndicator()),
    );
  }
}
