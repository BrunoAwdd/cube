import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'pages/upload_page.dart';
import 'pages/login_page.dart';
import 'pages/pairing_page.dart';
import 'pages/splash_page.dart';
import 'services/ws_service.dart';

void main() {
  runApp(
    ChangeNotifierProvider(
      create: (_) => WebSocketService(),
      child: const MyApp(),
    ),
  );
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      initialRoute: '/',
      routes: {
        '/': (context) => const SplashPage(),
        '/login': (context) => const LoginPage(),
        '/pair': (context) => const PairingPage(),
        '/gallery': (context) => const UploadPage(),
      },
    );
  }
}
