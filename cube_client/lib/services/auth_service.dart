import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';

class AuthService {
  static const _usernameKey = 'username';
  static const _tokenKey = 'auth_token';

  static Future<void> saveCredentials(String username, String token) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_usernameKey, username);
    await prefs.setString(_tokenKey, token);
  }

  static Future<Map<String, String>?> getCredentials() async {
    final prefs = await SharedPreferences.getInstance();
    final username = prefs.getString(_usernameKey);
    final token = prefs.getString(_tokenKey);
    if (username != null && token != null) {
      return {'username': username, 'token': token};
    }
    return null;
  }

  static Future<void> clearCredentials() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_usernameKey);
    await prefs.remove(_tokenKey);
  }

  static Future<String?> authenticate(String serverUrl, String code) async {
    try {
      final res = await http.post(
        Uri.parse("http://$serverUrl/auth"),
        headers: {"Content-Type": "application/json"},
        body: jsonEncode({"code": code}),
      );

      if (res.statusCode == 200) {
        final json = jsonDecode(res.body);
        return json['token'] as String;
      }
    } catch (e) {
      print("❌ Erro na autenticação: $e");
    }
    return null;
  }
}
