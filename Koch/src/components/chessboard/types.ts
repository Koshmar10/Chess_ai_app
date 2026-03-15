import { ChessPiece } from "../../../src-tauri/bindings/ChessPiece";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";
import { PieceType } from "../../../src-tauri/bindings/PieceType";

/** A piece tracked in the optimistic rendering layer */
export interface RenderedPiece {
  piece: ChessPiece;
  r: number;
  c: number;
  to_render: boolean;
}

/** A translucent "ghost" piece shown on hover previews */
export interface GhostPiece {
  kind: PieceType;
  color: PieceColor;
  r: number;
  c: number;
}

/** Arrow overlay data using numeric coordinate tuples */
export interface ArrowData {
  from: [number, number];
  to: [number, number];
  color?: string;
  type: "engine" | "user" | "ghost";
}

/** State for the promotion selection modal */
export type PromotionState = {
  from: [number, number];
  to: [number, number];
  color: PieceColor;
} | null;
