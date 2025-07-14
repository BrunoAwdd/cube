import React from "react";

export const ConnectionStatus: React.FC<{ online: boolean }> = ({ online }) => (
  <div
    style={{
      position: "absolute",
      top: 10,
      right: 20,
      display: "flex",
      alignItems: "center",
    }}
  >
    <div
      style={{
        width: 10,
        height: 10,
        borderRadius: "50%",
        backgroundColor: online ? "green" : "red",
        marginRight: 6,
      }}
    />
    <span style={{ fontSize: 12, color: "#555" }}>
      {online ? "Conectado" : "Offline"}
    </span>
  </div>
);
