import 'package:flutter/material.dart';
import 'pages/upload_page.dart';
import 'services/db_service.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await DbService.init();
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
