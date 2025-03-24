import 'package:sqflite/sqflite.dart';
import 'package:path/path.dart' as p;

class DbService {
  static late Database _db;

  static Future<void> init() async {
    final dbPath = await getDatabasesPath();
    _db = await openDatabase(
      p.join(dbPath, 'photos.db'),
      onCreate: (db, version) {
        return db.execute(
          'CREATE TABLE uploads(name TEXT, sha TEXT PRIMARY KEY, modified_at TEXT, updated_at TEXT)',
        );
      },
      version: 1,
    );
  }

  static Future<void> insertIfNotExists(String name, String sha, String modifiedAt) async {
    final existing = await _db.query('uploads', where: 'sha = ?', whereArgs: [sha]);
    if (existing.isEmpty) {
      await _db.insert('uploads', {
        'name': name,
        'sha': sha,
        'modified_at': modifiedAt,
        'updated_at': null,
      });
    }
  }

  static Future<void> updateAsSent(String sha) async {
    await _db.update(
      'uploads',
      {'updated_at': DateTime.now().toIso8601String()},
      where: 'sha = ?',
      whereArgs: [sha],
    );
  }

  static Future<List<Map<String, dynamic>>> getUnsent() {
    return _db.query('uploads', where: 'updated_at IS NULL');
  }

  static Future<void> clear() async {
    final dbPath = await getDatabasesPath();
    final path = p.join(dbPath, 'photos.db');
    await deleteDatabase(path);
  }
}
