import 'package:flutter/material.dart';
import 'package:photo_manager/photo_manager.dart';
import 'package:sqflite/sqflite.dart';
import 'package:path/path.dart' as p;
import 'package:http/http.dart' as http;
import 'package:crypto/crypto.dart';
import 'dart:typed_data';
import 'dart:convert';

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
      home: Scaffold(
        appBar: AppBar(
          title: const Text('Enviar Fotos pro PC'),
        ),
        body: const UploadPage(),
      ),
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

  @override
  void initState() {
    super.initState();
    initDbAndLoadPhotos();
  }

  Future<void> initDbAndLoadPhotos() async {
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

    final albums = await PhotoManager.getAssetPathList(type: RequestType.image);
    final recent = albums.first;
    final allAssets = await recent.getAssetListPaged(page: 0, size: 1000);

    for (final asset in allAssets) {
      final file = await asset.originFile;
      if (file == null) continue;

      final bytes = await file.readAsBytes();
      final hash = sha256.convert(bytes).toString();
      final name = file.path.split('/').last;
      final modifiedAt = asset.modifiedDateTime.toUtc().toIso8601String();
      print('modified_at: ${modifiedAt}');

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
    List<AssetEntity> matchedPhotos = [];

    for (final row in unsent) {
      for (final asset in allAssets) {
        final file = await asset.originFile;
        if (file == null) continue;
        final hash = sha256.convert(await file.readAsBytes()).toString();
        if (hash == row['sha']) {
          matchedPhotos.add(asset);
          break;
        }
      }
    }

    setState(() {
      photos = matchedPhotos;
    });
  }

  Future<void> deletePhotoDb() async {
    final dbPath = await getDatabasesPath();
    final path = p.join(dbPath, 'photos.db');
    await deleteDatabase(path);
  }

  Future<void> uploadPhotos() async {
    for (int i = 0; i < photos.length; i++) {
      final asset = photos[i];
      final file = await asset.originFile;
      if (file == null) continue;

      final request = http.MultipartRequest(
        'POST',
        Uri.parse('http://bruno-linux:8080/upload'),
      );

      request.files.add(await http.MultipartFile.fromPath('file', file.path));
      request.fields['modified_at'] = asset.modifiedDateTime.toUtc().toIso8601String();
      request.fields['updated_at'] = DateTime.now().toIso8601String();
      request.fields['username'] = "bruno"; // se ainda for relevante

      try {
        final response = await request.send();
        if (response.statusCode == 200) {
          final bytes = await file.readAsBytes();
          final hash = sha256.convert(bytes).toString();

          await db.update(
            'uploads',
            {'updated_at': DateTime.now().toIso8601String()},
            where: 'sha = ?',
            whereArgs: [hash],
          );

          print('✅ Enviada: ${file.path}');
        } else {
          print('❌ Falha ao enviar: ${file.path}, código ${response.statusCode}');
        }
      } catch (e) {
        print('❌ Erro ao enviar ${file.path}: $e');
      }
    }
    
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Upload finalizado com sucesso!')),
    );

    setState(() {
      photos.clear();
    });
  }


  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        ElevatedButton(
          onPressed: uploadPhotos,
          child: const Text('Enviar Fotos'),
        ),
        ElevatedButton(
          onPressed: () async {
            await deletePhotoDb();
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(content: Text('Banco deletado!')),
            );
          },
          child: const Text('Deletar DB'),
        ),
        ElevatedButton(
          onPressed: () async {
            await initDbAndLoadPhotos();
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(content: Text('Lista atualizada!')),
            );
          },
          child: const Text('Atualizar Lista'),
        ),
        Expanded(
          child: ListView.builder(
            itemCount: photos.length,
            itemBuilder: (context, index) {
              final asset = photos[index];
              return FutureBuilder<Uint8List?>(
                future: asset.thumbnailDataWithSize(const ThumbnailSize(200, 200)),
                builder: (context, snapshot) {
                  if (snapshot.connectionState != ConnectionState.done) {
                    return const ListTile(title: Text('Carregando...'));
                  }

                  final thumb = snapshot.data;
                  if (thumb == null) return const ListTile(title: Text('Erro ao carregar'));

                  return ListTile(
                    leading: Image.memory(thumb, fit: BoxFit.cover),
                    title: Text('Foto ${index + 1}'),
                  );
                },
              );
            },
          ),
        ),

      ],
    );
  }
}