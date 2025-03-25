import 'package:photo_manager/photo_manager.dart';
import 'package:http/http.dart' as http;
import 'package:crypto/crypto.dart';
import 'dart:io';

import 'db_service.dart';

class UploadService {
  static Future<void> uploadAll(
    List<AssetEntity> photos, {
    required void Function(String msg) onProgress,
    required void Function() onDone,
    required void Function(String error) onError,
  }) async {
    final db = DbService.db;

    for (final asset in photos) {
      final file = await asset.originFile;
      if (file == null) {
        print('⚠️ Ignorando arquivo nulo: ${asset.id}');
        continue;
      }

      try {
        final fileBytes = await file.readAsBytes();
        final hash = sha256.convert(fileBytes).toString();

        final uri = Uri.parse('http://bruno-linux:8080/upload_raw');

        final response = await http.post(
          uri,
          headers: {
            'Content-Type': 'application/octet-stream',
            'X-Filename': file.path.split('/').last,
            'X-Modified-At': asset.modifiedDateTime.toUtc().toIso8601String(),
            'X-Username': "bruno",
          },
          body: fileBytes,
        );

        if (response.statusCode == 200 && response.body.contains('Upload finalizado')) {
          await db.update(
            'uploads',
            {'updated_at': DateTime.now().toIso8601String()},
            where: 'sha = ?',
            whereArgs: [hash],
          );
          onProgress('✅ Enviada: ${file.path}');
        } else if (response.body.contains('existente')) {
          onProgress('⚠️ Já existia: ${file.path}');
        } else {
          onError('❌ Erro ${response.statusCode}: ${response.body}');
        }
      } catch (e) {
        onError('❌ Falha ao enviar ${file.path}: $e');
      }
    }

    onDone();
  }
}
