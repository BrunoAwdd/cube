import 'package:flutter/material.dart';
import 'package:photo_manager/photo_manager.dart';
import 'package:sqflite/sqflite.dart';
import 'package:path/path.dart' as p;
import 'package:http/http.dart' as http;
import 'package:crypto/crypto.dart';
import 'dart:typed_data';
import 'dart:convert';
import 'dart:io';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await PhotoManager.requestPermissionExtend();
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

class UploadPage extends StatefulWidget {
  const UploadPage({super.key});

  @override
  State<UploadPage> createState() => _UploadPageState();
}

class _UploadPageState extends State<UploadPage> {
  late Database db;
  List<AssetEntity> photos = [];
  bool isUploading = false;

  @override
  void initState() {
    super.initState();
    _initializeApp();
  }

  Future<void> _initializeApp() async {
    await _openDatabase();
    await _loadPhotos();
  }

  Future<void> _openDatabase() async {
    final dbPath = await getDatabasesPath();
    db = await openDatabase(
      p.join(dbPath, 'photos.db'),
      onCreate: (db, version) {
        return db.execute(
          'CREATE TABLE uploads(name TEXT, sha TEXT PRIMARY KEY, modified_at TEXT, updated_at TEXT)',
        );
      },
      version: 1,
    );
  }

  Future<void> _loadPhotos() async {
    final albums = await PhotoManager.getAssetPathList(type: RequestType.image);
    if (albums.isEmpty) return;

    final allAssets = await albums.first.getAssetListPaged(page: 0, size: 1000);

    for (final asset in allAssets) {
      final file = await asset.originFile;
      if (file == null) continue;

      final bytes = await file.readAsBytes();
      final hash = sha256.convert(bytes).toString();
      final name = file.path.split('/').last;
      final modifiedAt = asset.modifiedDateTime.toUtc().toIso8601String();

      final existing = await db.query('uploads', where: 'sha = ?', whereArgs: [hash]);
      if (existing.isEmpty) {
        await db.insert('uploads', {
          'name': name,
          'sha': hash,
          'modified_at': modifiedAt,
          'updated_at': null,
        });
      }
    }

    final unsent = await db.query('uploads', where: 'updated_at IS NULL');
    List<AssetEntity> matched = [];

    for (final row in unsent) {
      for (final asset in allAssets) {
        final file = await asset.originFile;
        if (file == null) continue;
        final hash = sha256.convert(await file.readAsBytes()).toString();
        if (hash == row['sha']) {
          matched.add(asset);
          break;
        }
      }
    }

    setState(() {
      photos = matched;
    });
  }

  Future<void> _deleteDatabase() async {
    final path = p.join(await getDatabasesPath(), 'photos.db');
    await deleteDatabase(path);
  }

  Future<void> _uploadPhotos() async {
    setState(() => isUploading = true);

    for (final asset in photos) {
      final file = await asset.originFile;
      if (file == null) continue;

      final fileBytes = await file.readAsBytes();
      final hash = sha256.convert(fileBytes).toString();

      final request = http.MultipartRequest(
        'POST',
        Uri.parse('http://bruno-linux:8080/upload'),
      )
        ..files.add(http.MultipartFile.fromBytes(
          'file',
          fileBytes,
          filename: file.path.split('/').last,
        ))
        ..fields['modified_at'] = asset.modifiedDateTime.toUtc().toIso8601String()
        ..fields['updated_at'] = DateTime.now().toIso8601String()
        ..fields['username'] = 'bruno';

      try {
        final response = await request.send();
        if (response.statusCode == 200) {
          await db.update(
            'uploads',
            {'updated_at': DateTime.now().toIso8601String()},
            where: 'sha = ?',
            whereArgs: [hash],
          );
          print('✅ Enviada: ${file.path}');
        } else {
          print('❌ Falha: ${file.path} [${response.statusCode}]');
        }
      } catch (e) {
        print('❌ Erro ao enviar ${file.path}: $e');
      }
    }

    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Upload finalizado com sucesso!')),
    );

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
              await _deleteDatabase();
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Banco deletado!')),
              );
            },
            child: const Text('Deletar DB'),
          ),
          ElevatedButton(
            onPressed: () async {
              await _loadPhotos();
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
                    if (!snapshot.hasData) {
                      return const ListTile(title: Text('Carregando...'));
                    }
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
