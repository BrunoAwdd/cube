import 'package:photo_manager/photo_manager.dart';
import 'package:crypto/crypto.dart';

import 'db_service.dart';

class PhotoService {
  static Future<List<AssetEntity>> loadUnsentPhotos({limit= 5}) async {    
    final albums = await PhotoManager.getAssetPathList(type: RequestType.image);
    final recent = albums.first;
    final assets = await recent.getAssetListPaged(page: 0, size: limit);

    for (final asset in assets) {
      final file = await asset.originFile;
      if (file == null) continue;
      final bytes = await file.readAsBytes();
      final sha = sha256.convert(bytes).toString();
      final name = file.path.split('/').last;
      final modified = asset.modifiedDateTime.toUtc().toIso8601String();
      await DbService.insertIfNotExists(name, sha, modified);
    }

    final unsent = await DbService.getUnsent();
    List<AssetEntity> matched = [];

    for (final row in unsent) {
      for (final asset in assets) {
        final file = await asset.originFile;
        if (file == null) continue;
        final hash = sha256.convert(await file.readAsBytes()).toString();
        if (hash == row['sha']) {
          matched.add(asset);
          break;
        }
      }
    }

    return matched;
  }
}
