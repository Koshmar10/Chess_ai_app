import React from "react";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";
import { PieceType } from "../../../src-tauri/bindings/PieceType";
import { getImage } from "./utils";

const PROMOTION_PIECES: PieceType[] = ["Queen", "Rook", "Bishop", "Knight"];

interface PromotionModalProps {
  color: PieceColor;
  onSelect: (pieceType: PieceType) => void;
}

export const PromotionModal = React.memo(function PromotionModal({
  color,
  onSelect,
}: PromotionModalProps) {
  return (
    <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/40 rounded-sm backdrop-blur-[1px]">
      <div className="bg-zinc-900/90 p-3 rounded-lg shadow-2xl border border-zinc-700 flex gap-3 animate-in fade-in zoom-in duration-200">
        {PROMOTION_PIECES.map(kind => (
          <button
            key={kind}
            className="p-2 hover:bg-zinc-700/50 rounded-md flex flex-col items-center justify-center transition-all hover:scale-110 active:scale-95"
            onClick={() => onSelect(kind)}
            title={`Promote to ${kind}`}
          >
            <img
              src={getImage(color, kind)}
              alt={kind}
              className="w-8 h-8 object-cover"
            />
            <span className="text-xs font-semibold text-white">{kind}</span>
          </button>
        ))}
      </div>
    </div>
  );
});
