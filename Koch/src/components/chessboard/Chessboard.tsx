import { Board } from "../../../src-tauri/bindings/Board";
import { BoardSquare } from "./BoardSquare";
import { ArrowLayer } from "../ArrowLayer";
import { PieceColor } from "../../../src-tauri/bindings/PieceColor";
import { PieceLayer } from "./PieceLayer";
import { PromotionModal } from "./PromotionModal";
import { useChessInteraction } from "../../hooks/useChessInteraction";
import { useInfluenceTint } from "../../hooks/useInfluenceTint";
import { useCheckSquares } from "../../hooks/useCheckSquares";
import { ArrowData, GhostPiece } from "./types";

// Re-export types for backward compatibility with existing consumer imports
export type { RenderedPiece, GhostPiece, ArrowData, PromotionState } from "./types";

const FILES = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

interface ChessBoardProps {
  squareSize: number;
  board: Board;
  onMove?: (from: [number, number], to: [number, number], promotion?: string) => void;
  lastMove?: [number[], number[]] | null;
  flipped?: boolean;
  isInteractive?: boolean;
  suggestion?: ArrowData | null;
  threat?: ArrowData | null;
  tintEnabled?: boolean;
  hoverTarget?: [number, number] | null;
  ghostPieces?: GhostPiece[];
  ghostArrows?: ArrowData[];
  user?: PieceColor;
  wasCorrectPuzzleMove?: boolean | null;
}

export function ChessBoard({
  squareSize,
  board,
  onMove,
  lastMove,
  flipped = false,
  suggestion = null,
  threat = null,
  tintEnabled = false,
  hoverTarget = null,
  ghostPieces = [],
  ghostArrows = [],
  user,
  wasCorrectPuzzleMove = null,
}: ChessBoardProps) {

  // Interaction hook manages selection, moves, optimistic pieces, arrows, and promotion
  const interaction = useChessInteraction({
    board,
    onMove,
    user,
    lastMove,
  });

  // Precomputed influence tint map — single O(pieces*moves) pass instead of O(64*pieces*moves)
  const tintMap = useInfluenceTint(board, interaction.optimisticPieces, tintEnabled);

  // Precomputed check squares — single pass instead of 64 per-square calls
  const checkSquares = useCheckSquares(board);

  const getLogicalCoords = (visualRow: number, visualCol: number) => {
    return flipped ? [7 - visualRow, 7 - visualCol] : [visualRow, visualCol];
  };

  return (
    <div className="relative inline-block" onContextMenu={interaction.handleContextMenu}>
      {/* Board squares */}
      <div
        className="grid relative z-0"
        style={{
          gridTemplateColumns: `repeat(8, ${squareSize}px)`,
          gridTemplateRows: `repeat(8, ${squareSize}px)`,
          width: squareSize * 8,
          height: squareSize * 8,
        }}
      >
        {Array.from({ length: 64 }, (_, i) => {
          const visualRow = Math.floor(i / 8);
          const visualCol = i % 8;
          const [row, col] = getLogicalCoords(visualRow, visualCol);
          const isDark = (visualRow + visualCol) % 2 === 1;

          // When tint is enabled, hide selected/last-move highlights
          const isSelected = tintEnabled
            ? false
            : (
              (!!interaction.selectedSquare && interaction.selectedSquare[0] === row && interaction.selectedSquare[1] === col) ||
              (!!interaction.lastMoveMade &&
                ((interaction.lastMoveMade[0][0] === row && interaction.lastMoveMade[0][1] === col) ||
                  (interaction.lastMoveMade[1][0] === row && interaction.lastMoveMade[1][1] === col)))
            );

          // Look up precomputed tint color
          const tintColor = tintMap?.get(`${row}-${col}`);

          // Annotation Logic
          const isLeftEdge = visualCol === 0;
          const isBottomEdge = visualRow === 7;
          const rankLabel = isLeftEdge ? (flipped ? visualRow + 1 : 8 - visualRow) : null;
          const fileLabel = isBottomEdge ? (flipped ? FILES[7 - visualCol] : FILES[visualCol]) : null;

          return (
            <div
              key={`${row}-${col}`}
              className="relative p-0 m-0"
              style={{ width: squareSize, height: squareSize }}
            >
              <BoardSquare
                dark={isDark}
                selected={isSelected}
                puzzleMoveCorrect={wasCorrectPuzzleMove}
                tint={tintColor}
                size={squareSize}
                attackMove={false}
                hover={hoverTarget ? hoverTarget[0] === row && hoverTarget[1] === col : false}
                inCheck={checkSquares.has(`${row}-${col}`)}
                captureMove={!!interaction.selectedMoves && interaction.selectedMoves.capture_moves.some((m) => m[0] === row && m[1] === col)}
                quietMove={!!interaction.selectedMoves && interaction.selectedMoves.quiet_moves.some((m) => m[0] === row && m[1] === col)}
                handleMouseDown={(e) => interaction.onSquareMouseDown(row, col, e)}
                handleMouseUp={(e) => interaction.onSquareMouseUp(row, col, e)}
              />

              {/* Rank Annotation (Top-Left) */}
              {rankLabel && (
                <span className={`absolute top-0.5 left-1 text-[10px] font-bold select-none pointer-events-none ${isDark ? "text-white/60" : "text-black/60"}`}>
                  {rankLabel}
                </span>
              )}

              {/* File Annotation (Bottom-Right) */}
              {fileLabel && (
                <span className={`absolute bottom-0 right-1 text-[10px] font-bold select-none pointer-events-none ${isDark ? "text-white/60" : "text-black/60"}`}>
                  {fileLabel}
                </span>
              )}
            </div>
          );
        })}
      </div>

      {/* Pieces layer on top */}
      <PieceLayer flipped={flipped} optimisticPieces={interaction.optimisticPieces} squareSize={squareSize} ghostPieces={ghostPieces} />
      <ArrowLayer
        arrows={interaction.arrows}
        suggestion={suggestion}
        threat={threat}
        ghostArrows={ghostArrows}
        isFlipped={flipped}
        squareSize={squareSize}
      />

      {/* Promotion Modal */}
      {interaction.promotionMove && (
        <PromotionModal
          color={interaction.promotionMove.color}
          onSelect={interaction.handlePromotionSelect}
        />
      )}
    </div>
  );
}
