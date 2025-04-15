import 'package:photo_manager/photo_manager.dart';
import 'package:http/http.dart' as http;
import 'package:crypto/crypto.dart';
import 'dart:io';

import 'db_service.dart';

class UploadService {
  static Future<void> uploadAll(
    List<AssetEntity> photos, {
    required void Function(String msg, String assetId) onProgress,
    required void Function(String assetId) onSuccess,
    required void Function(String error, String assetId) onError,
    required void Function() onDone,
  }) async {
    final db = DbService.db;

    for (final asset in photos) {
      final file = await asset.originFile;
      if (file == null) {
        print('⚠️ Ignorando arquivo nulo: ${asset.id}');
        continue;
      }

      try {
        onProgress('📤 Enviando ${file.path}', asset.id);

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
          onSuccess(asset.id);
        } else if (response.body.contains('existente')) {
          onSuccess(asset.id); // mesmo status de sucesso para arquivos já existentes
        } else {
          onError('❌ Erro ${response.statusCode}: ${response.body}', asset.id);
        }
      } catch (e) {
        onError('❌ Falha ao enviar ${file.path}: $e', asset.id);
      }
    }

    onDone();
  }
  static Future<void> uploadSingle(
    AssetEntity asset, {
    String username = "bruno",
  }) async {
    final file = await asset.originFile;
    if (file == null) {
      print("⚠️ Arquivo nulo para ${asset.id}");
      return;
    }

    final fileBytes = await file.readAsBytes();
    final hash = sha256.convert(fileBytes).toString();

    final uri = Uri.parse('http://bruno-linux:8080/upload_raw');

    final response = await http.post(
      uri,
      headers: {
        'Content-Type': 'application/octet-stream',
        'X-Filename': file.path.split('/').last,
        'X-Modified-At': asset.modifiedDateTime.toUtc().toIso8601String(),
        'X-Username': username,
      },
      body: fileBytes,
    );

    if (response.statusCode == 200 && response.body.contains('Upload finalizado')) {
      print("✅ Upload concluído para $hash");
    } else if (response.body.contains('existente')) {
      print("ℹ️ Arquivo já existente: $hash");
    } else {
      print("❌ Falha ao enviar $hash: ${response.body}");
    }
  }

}
