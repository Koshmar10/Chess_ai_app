import React from "react";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";
import { PieceType } from "../../../src-tauri/bindings/PieceType";
import { getImage } from "./utils";

interface PieceProps {
  size: number;
  row: number;
  col: number;
  color: PieceColor;
  kind: PieceType;
  is_ghost?: boolean;
}

export const Piece = React.memo(function Piece({ size, row, col, color, kind, is_ghost = false }: PieceProps) {
  return (
    <img
      src={getImage(color, kind)}
      alt=""
      className={`absolute ${is_ghost ? 'opacity-60' : 'opacity-100'}`}
      style={{
        zIndex: 10,
        width: size * 0.8,
        height: size * 0.8,
        top: 0,
        left: 0,
        transform: `translate3d(${col * size + size * 0.1}px, ${row * size + size * 0.1}px, 0)`,
        transition: "transform 250ms ease-in-out",
        willChange: 'transform',
      }}
      draggable={false}
    />
  );
});
