import { useMemo } from "react";
import { Board } from "../../src-tauri/bindings/Board";
import { PieceColor } from "../../src-tauri/bindings/PieceColor";

/**
 * Precomputes which squares have a king in check.
 * Returns a Set<string> of "row-col" keys for squares where the king is in check.
 *
 * This replaces the previous per-square isInCheck() that rebuilt a full
 * piece-color map 64 times per render with a single O(pieces) computation.
 */
export function useCheckSquares(board: Board): Set<string> {
  return useMemo(() => {
    const checkSquares = new Set<string>();

    // Find king positions
    const kings: { r: number; c: number; color: PieceColor }[] = [];
    const idToColor = new Map<number, PieceColor>();

    for (let r = 0; r < 8; r++) {
      for (let c = 0; c < 8; c++) {
        const p = board.squares[r][c];
        if (!p) continue;
        idToColor.set(p.id, p.color);
        if (p.kind === "King") {
          kings.push({ r, c, color: p.color });
        }
      }
    }

    // For each king, check if any opponent piece attacks its square
    for (const king of kings) {
      for (const [idStr, moves] of Object.entries(board.move_cache)) {
        const id = Number(idStr);
        const attackerColor = idToColor.get(id);
        if (!attackerColor || attackerColor === king.color) continue;

        if (moves && moves.capture_moves.some(([mr, mc]) => mr === king.r && mc === king.c)) {
          checkSquares.add(`${king.r}-${king.c}`);
          break; // This king is in check, no need to check more attackers
        }
      }
    }

    return checkSquares;
  }, [board]);
}
