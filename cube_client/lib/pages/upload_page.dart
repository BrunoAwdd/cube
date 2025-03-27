import 'package:flutter/material.dart';
import 'package:photo_manager/photo_manager.dart';
import 'dart:typed_data';
import 'package:provider/provider.dart';

import '../services/ws_service.dart';
import '../services/db_service.dart';
import '../services/photo_service.dart';
import '../services/upload_service.dart';
import '../services/send_thumbnails.dart';

class UploadPage extends StatefulWidget {
  const UploadPage({super.key});

  @override
  State<UploadPage> createState() => _UploadPageState();
}

class _UploadPageState extends State<UploadPage> {
  List<AssetEntity> photos = [];
  bool isUploading = false;
  final Map<String, String> _statusMap = {}; // asset.id => status

  @override
  void initState() {
    super.initState();
    final ws = Provider.of<WebSocketService>(context, listen: false);
    ws.setOnSessionLost(() {
      if (mounted) {
        Navigator.pushReplacementNamed(context, '/pair');
      }
    });
    _setup();
  }


  Future<void> _setup() async {
    await DbService.init();
    final assets = await PhotoService.loadUnsentPhotos(limit: 50);
    setState(() => photos = assets);
    await sendThumbnailsToRust(photos);
  }

  Future<void> _uploadPhotos() async {
    setState(() => isUploading = true);

    await UploadService.uploadAll(
      photos,
      onProgress: (path, assetId) {
        setState(() => _statusMap[assetId] = 'uploading');
      },
      onSuccess: (assetId) {
        setState(() => _statusMap[assetId] = 'success');
      },
      onError: (msg, assetId) {
        print('❌ Erro: $msg');
        setState(() => _statusMap[assetId] = 'error');
      },
      onDone: () {
        setState(() => isUploading = false);
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('✅ Upload finalizado com sucesso!')),
        );
      },
    );
  }

  Widget _buildStatusIcon(String? status) {
    switch (status) {
      case 'success':
        return Icon(Icons.check_circle, color: Colors.green, size: 20);
      case 'error':
        return Icon(Icons.error, color: Colors.red, size: 20);
      case 'uploading':
        return const SizedBox(
          width: 20,
          height: 20,
          child: CircularProgressIndicator(strokeWidth: 2),
        );
      default:
        return const SizedBox.shrink();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Enviar Fotos pro PC'), actions: [
        Consumer<WebSocketService>(
          builder: (context, ws, _) => Padding(
            padding: const EdgeInsets.only(right: 16),
            child: Row(
              children: [
                Icon(
                  Icons.circle,
                  size: 10,
                  color: ws.isConnected ? Colors.green : Colors.red,
                ),
                const SizedBox(width: 4),
                Text(
                  ws.isConnected ? 'Conectado' : 'Offline',
                  style: const TextStyle(fontSize: 12),
                ),
              ],
            ),
          ),
        ),
      ],),
      body: Column(
        children: [
          const SizedBox(height: 10),
          Wrap(
            spacing: 10,
            children: [
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
                  final assets = await PhotoService.loadUnsentPhotos(limit: 50);
                  setState(() => photos = assets);
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('Lista atualizada!')),
                  );
                },
                child: const Text('Atualizar Lista'),
              ),
            ],
          ),
          const Divider(),
          Expanded(
            child: GridView.builder(
              padding: const EdgeInsets.all(8),
              gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                crossAxisCount: 4,
                crossAxisSpacing: 6,
                mainAxisSpacing: 6,
              ),
              itemCount: photos.length,
              itemBuilder: (context, index) {
                final asset = photos[index];
                final status = _statusMap[asset.id];

                return Stack(
                  children: [
                    Positioned.fill(
                      child: FutureBuilder<Uint8List?>(
                        future: asset.thumbnailDataWithSize(const ThumbnailSize(300, 300)),
                        builder: (context, snapshot) {
                          if (!snapshot.hasData) {
                            return Container(color: Colors.black12);
                          }
                          return ClipRRect(
                            borderRadius: BorderRadius.circular(8),
                            child: Image.memory(snapshot.data!, fit: BoxFit.cover),
                          );
                        },
                      ),
                    ),
                    Positioned(
                      top: 6,
                      right: 6,
                      child: _buildStatusIcon(status),
                    ),
                  ],
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
