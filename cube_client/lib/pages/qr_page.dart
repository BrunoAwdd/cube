import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';

class QrPage extends StatelessWidget {
  final String serverAddress;

  const QrPage({super.key, required this.serverAddress});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('QR Code do Servidor')),
      body: Center(
        child: QrImageView(
          data: serverAddress,
          version: QrVersions.auto,
          size: 300.0,
          gapless: false,
        ),
      ),
    );
  }
}
