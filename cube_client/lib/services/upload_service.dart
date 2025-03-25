import 'package:photo_manager/photo_manager.dart';
import 'package:http/http.dart' as http;
import 'package:crypto/crypto.dart';

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

      request.files.add(http.MultipartFile.fromBytes(
        'file',
        fileBytes,
        filename: file.path.split('/').last,
      ));

      request.fields['modified_at'] = asset.modifiedDateTime.toUtc().toIso8601String();
      request.fields['updated_at'] = DateTime.now().toIso8601String();
      request.fields['username'] = "bruno";

      try {
        onProgress('📤 Enviando ${file.path}');
        final response = await request.send();

        if (response.statusCode == 200) {
          await db.update(
            'uploads',
            {'updated_at': DateTime.now().toIso8601String()},
            where: 'sha = ?',
            whereArgs: [hash],
          );
        } else {
          onError('❌ Erro ${response.statusCode} ao enviar ${file.path}');
        }
      } catch (e) {
        onError('❌ Falha ao enviar ${file.path}: $e');
      }
    }

    onDone();
  }
}
