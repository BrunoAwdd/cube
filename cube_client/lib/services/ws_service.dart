// lib/services/ws_service.dart
import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

class WebSocketService with ChangeNotifier {
  WebSocketChannel? _channel;
  bool _connected = false;
  bool _connecting = false;
  String? _ip;

  Function()? _onSessionLost;
  final List<void Function(Map<String, dynamic>)> _listeners = [];

  bool get isConnected => _connected;
  String? get currentIp => _ip;

  void setOnSessionLost(Function() callback) {
    _onSessionLost = callback;
  }

  void connect(String ip) {
    _ip = ip;

    if (_connecting || _connected) return;
    _connecting = true;
    notifyListeners();

    final url = 'ws://$ip:8080/ws';
    print("📲 Conectando ao WebSocket: $url - $ip");
    try {
      _channel = WebSocketChannel.connect(Uri.parse(url));
    } catch (e) {
      print('❌ Falha ao conectar: $e');
      _scheduleReconnect();
      return;
    }

    _channel!.stream.listen(
    (message) {
      print('📩 WS mensagem recebida: $message');
      try {
        final data = jsonDecode(message);
        if (data is Map<String, dynamic>) {
          if (!_connected) {
            print('🟢 Conectado (mensagem válida recebida)');
            _connected = true;
            _connecting = false;
            notifyListeners();
          }

          for (final listener in _listeners) {
            listener(data);
          }
        }
      } catch (e) {
        print('❌ Erro ao decodificar WS: $e');
      }
    },
    onError: (error) {
      print('❌ Erro no WebSocket: $error');
      _handleDisconnect();
    },
    onDone: () {
      print('🟡 WebSocket desconectado');
      _handleDisconnect();
    },
  );

  // ✅ Acrescente essa parte após conectar com sucesso
  _connected = true;
  _connecting = false;
  notifyListeners();
  }



  void _handleDisconnect() {
    _connected = false;
    _connecting = false;
    notifyListeners();

    _scheduleReconnect();

    if (_onSessionLost != null) {
      _onSessionLost!();
    }
  }

  void _scheduleReconnect() {
    if (_ip != null) {
      Future.delayed(const Duration(seconds: 5), () {
        if (!_connected && !_connecting) {
          print("🔁 Tentando reconectar...");
          connect(_ip!);
        }
      });
    }
  }

  void send(Map<String, dynamic> message) {
    if (_connected && _channel != null) {
      _channel!.sink.add(jsonEncode(message));
    } else {
      print("❌ Tentando enviar WS sem conexão.");
    }
  }

  void disconnect() {
    _channel?.sink.close();
    _connected = false;
    notifyListeners();
  }

  void addListenerCallback(void Function(Map<String, dynamic>) callback) {
    _listeners.add(callback);
  }
}
