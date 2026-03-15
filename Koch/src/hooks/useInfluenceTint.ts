import { useMemo } from "react";
import { Board } from "../../src-tauri/bindings/Board";
import { RenderedPiece } from "../components/chessboard/types";

/**
 * Precomputes an influence tint color map for all 64 squares in a single pass.
 * Returns a Map<string, string> keyed by "row-col" with RGBA color values,
 * or null if tint is disabled.
 *
 * This replaces the previous O(64 * pieces * moves) per-render computation
 * with a single O(pieces * moves) pass.
 */
export function useInfluenceTint(
  board: Board,
  optimisticPieces: RenderedPiece[],
  tintEnabled: boolean
): Map<string, string> | null {
  return useMemo(() => {
    if (!tintEnabled) return null;

    // Accumulate per-square influence counts in a single pass
    const whiteInfluence = new Int8Array(64); // flat [row * 8 + col]
    const blackInfluence = new Int8Array(64);

    // Build a quick id -> color lookup from optimistic pieces
    const idToColor = new Map<number, string>();
    for (const rp of optimisticPieces) {
      if (rp.to_render) {
        idToColor.set(rp.piece.id, rp.piece.color);
      }
    }

    // Single pass over move_cache
    for (const [idStr, moves] of Object.entries(board.move_cache)) {
      if (!moves || !Array.isArray(moves.attacks)) continue;
      const id = Number(idStr);
      const owner = idToColor.get(id);
      if (!owner) continue;

      for (const [mr, mc] of moves.attacks) {
        const idx = mr * 8 + mc;
        if (owner === "White") {
          whiteInfluence[idx]++;
        } else {
          blackInfluence[idx]++;
        }
      }
    }

    // Now compute colors for each square
    const map = new Map<string, string>();

    const computeAlpha = (diff: number, baseMin = 0.12, baseMaxDelta = 0.38): number => {
      const intensity = Math.min(diff, 7) / 7;
      return baseMin + intensity * baseMaxDelta;
    };

    for (let row = 0; row < 8; row++) {
      for (let col = 0; col < 8; col++) {
        const idx = row * 8 + col;
        const w = whiteInfluence[idx];
        const b = blackInfluence[idx];

        if (w === 0 && b === 0) continue;

        const isDark = (row + col) % 2 === 1;
        const opacityFactor = isDark ? 0.72 : 1.25;

        let tintColor: string;

        if (w > b) {
          const diff = w - b;
          let alpha = computeAlpha(diff);
          alpha = Math.max(0.03, Math.min(1, alpha * opacityFactor));
          tintColor = `rgba(37,99,235,${alpha.toFixed(3)})`;
        } else if (b > w) {
          const diff = b - w;
          let alpha = computeAlpha(diff);
          alpha = Math.max(0.03, Math.min(1, alpha * opacityFactor));
          tintColor = `rgba(220,38,38,${alpha.toFixed(3)})`;
        } else {
          // equal non-zero -> contested/purple
          let alpha = computeAlpha(Math.min(w, 7), 0.10, 0.35);
          alpha = Math.max(0.03, Math.min(1, alpha * opacityFactor));
          tintColor = `rgba(128,0,128,${alpha.toFixed(3)})`;
        }

        map.set(`${row}-${col}`, tintColor);
      }
    }

    return map;
  }, [board.move_cache, optimisticPieces, tintEnabled]);
}
