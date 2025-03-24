import 'package:flutter/material.dart';
import 'pages/upload_page.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const PhotoUploaderApp());
}

class PhotoUploaderApp extends StatelessWidget {
  const PhotoUploaderApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Uploader de Fotos',
      home: const UploadPage(),
    );
  }
}
