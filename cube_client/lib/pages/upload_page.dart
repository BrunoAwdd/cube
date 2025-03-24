import 'package:flutter/material.dart';
import 'package:photo_manager/photo_manager.dart';
import 'dart:typed_data';

import '../services/db_service.dart';
import '../services/photo_service.dart';
import '../services/upload_service.dart';

class UploadPage extends StatefulWidget {
  const UploadPage({super.key});

  @override
  State<UploadPage> createState() => _UploadPageState();
}

class _UploadPageState extends State<UploadPage> {
  List<AssetEntity> photos = [];
  bool isUploading = false;

  @override
  void initState() {
    super.initState();
    _setup();
  }

  Future<void> _setup() async {
    await DbService.init();
    final assets = await PhotoService.loadUnsentPhotos();
    setState(() => photos = assets);
  }

  Future<void> _uploadPhotos() async {
    setState(() => isUploading = true);
    await UploadService.uploadAll(photos);
    setState(() {
      isUploading = false;
      photos.clear();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Enviar Fotos pro PC')),
      body: Column(
        children: [
          const SizedBox(height: 10),
          ElevatedButton(
            onPressed: isUploading ? null : _uploadPhotos,
            child: Text(isUploading ? 'Enviando...' : 'Enviar Fotos'),
          ),
          ElevatedButton(
            onPressed: () async {
              await DbService.clear();
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Banco deletado!')),
              );
            },
            child: const Text('Deletar DB'),
          ),
          ElevatedButton(
            onPressed: () async {
              final assets = await PhotoService.loadUnsentPhotos();
              setState(() => photos = assets);
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Lista atualizada!')),
              );
            },
            child: const Text('Atualizar Lista'),
          ),
          const Divider(),
          Expanded(
            child: ListView.builder(
              itemCount: photos.length,
              itemBuilder: (context, index) {
                final asset = photos[index];
                return FutureBuilder<Uint8List?>(
                  future: asset.thumbnailDataWithSize(const ThumbnailSize(200, 200)),
                  builder: (context, snapshot) {
                    if (!snapshot.hasData) return const ListTile(title: Text('Carregando...'));
                    return ListTile(
                      leading: Image.memory(snapshot.data!, fit: BoxFit.cover),
                      title: Text('Foto ${index + 1}'),
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
