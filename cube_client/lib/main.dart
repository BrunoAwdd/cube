import 'package:flutter/material.dart';
import 'pages/upload_page.dart';
import 'pages/login_page.dart';
import 'pages/pairing_page.dart';
import 'pages/splash_page.dart';

void main() {
  runApp(MaterialApp(
    initialRoute: '/',
    routes: {
      '/': (context) => const SplashPage(),
      '/login': (context) => const LoginPage(),
      '/pair': (context) => const PairingPage(),
      '/gallery': (context) => const UploadPage(), 
    },
  ));
}


