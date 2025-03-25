import 'package:photo_manager/photo_manager.dart';
import 'package:http/http.dart' as http;
import 'package:crypto/crypto.dart';
import 'package:http_parser/http_parser.dart';

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
      if (file == null) continue;

      final fileBytes = await file.readAsBytes();
      final hash = sha256.convert(fileBytes).toString();

      final request = http.MultipartRequest(
        'POST',
        Uri.parse('http://bruno-linux:8080/upload'),
      );

      final extension = file.path.split('.').last.toLowerCase();
      final mimeType = extension == 'heic'
          ? MediaType('image', 'heic')
          : MediaType('image', 'jpeg'); // fallback

      request.files.add(http.MultipartFile.fromBytes(
        'file',
        fileBytes,
        filename: file.path.split('/').last,
        contentType: mimeType,
      ));


      request.fields['modified_at'] = asset.modifiedDateTime.toUtc().toIso8601String();
      request.fields['updated_at'] = DateTime.now().toIso8601String();
      request.fields['username'] = "bruno";

      try {
        onProgress('📤 Enviando ${file.path}');
        final response = await request.send();

        final body = await response.stream.bytesToString(); // lê a resposta completa

        if (response.statusCode == 200 && body.contains('Upload finalizado')) {
          await db.update(
            'uploads',
            {'updated_at': DateTime.now().toIso8601String()},
            where: 'sha = ?',
            whereArgs: [hash],
          );
          onProgress('✅ Enviada: ${file.path}');
        } else {
          onError('❌ Erro ${response.statusCode}: $body');
        }
      } catch (e) {
        onError('❌ Falha ao enviar ${file.path}: $e');
      }
    }

    onDone();
  }
}
