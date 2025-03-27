import 'dart:convert';
import 'dart:typed_data';
import 'package:photo_manager/photo_manager.dart';
import 'package:http/http.dart' as http;
import 'package:crypto/crypto.dart';

Future<void> sendThumbnailsToRust(List<AssetEntity> photos, String serverIp) async {
  final List<Map<String, dynamic>> payload = [];

  for (final asset in photos) {
    final file = await asset.originFile;
    if (file == null) continue;

    final thumb = await asset.thumbnailDataWithSize(const ThumbnailSize(200, 200));
    if (thumb == null) continue;

    final bytes = await file.readAsBytes();
    final hash = sha256.convert(bytes).toString();
    final sizeMB = (bytes.length / (1024 * 1024)).toStringAsFixed(2);

    payload.add({
      "id": asset.id,
      "name": file.path.split("/").last,
      "size": "$sizeMB MB",
      "thumb_base64": base64Encode(thumb),
      "hash": hash,
      "status": "idle",
      "modified_at": asset.modifiedDateTime.toUtc().toIso8601String(),
    });
  }

  try {
    final response = await http.post(
      Uri.parse("http://$serverIp:8080/thumbs"),
      headers: {"Content-Type": "application/json"},
      body: jsonEncode(payload),
    );

    if (response.statusCode == 200) {
      print("✅ Thumbnails enviados com sucesso");
    } else {
      print("❌ Erro: ${response.statusCode} - ${response.body}");
    }
  } catch (e) {
    print("❌ Falha no envio: $e");
  }
}

