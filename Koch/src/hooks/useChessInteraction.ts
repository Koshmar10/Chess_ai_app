import { useEffect, useState, useCallback } from "react";
import { Board } from "../../src-tauri/bindings/Board";
import { PieceMoves } from "../../src-tauri/bindings/PieceMoves";
import { PieceColor } from "../../src-tauri/bindings/PieceColor";
import { PieceType } from "../../src-tauri/bindings/PieceType";
import { RenderedPiece, PromotionState } from "../components/chessboard/types";
import { ArrowData } from "../components/chessboard/types";

export interface UseChessInteractionConfig {
  board: Board;
  onMove?: (from: [number, number], to: [number, number], promotion?: string) => void;
  user?: PieceColor;
  lastMove?: [number[], number[]] | null;
}

export interface ChessInteraction {
  // State
  selectedSquare: number[] | null;
  selectedMoves: PieceMoves | null;
  optimisticPieces: RenderedPiece[];
  arrows: ArrowData[];
  lastMoveMade: [number[], number[]] | null;
  promotionMove: PromotionState;

  // Handlers
  onSquareMouseDown: (r: number, c: number, e: React.MouseEvent) => void;
  onSquareMouseUp: (r: number, c: number, e: React.MouseEvent) => void;
  handlePromotionSelect: (pieceType: PieceType) => void;
  handleContextMenu: (e: React.MouseEvent) => void;

  // Utilities
  isLegalMove: (r: number, c: number) => boolean;
}

export function useChessInteraction({
  board,
  onMove,
  user,
  lastMove,
}: UseChessInteractionConfig): ChessInteraction {
  const [selectedSquare, setSelectedSquare] = useState<number[] | null>(null);
  const [selectedMoves, setSelectedMoves] = useState<PieceMoves | null>(null);
  const [arrows, setArrows] = useState<ArrowData[]>([]);
  const [startArrow, setStartArrow] = useState<number[] | null>(null);
  const [lastMoveMade, setLastMoveMade] = useState<[number[], number[]] | null>(lastMove || null);
  const [promotionMove, setPromotionMove] = useState<PromotionState>(null);
  const [optimisticPieces, setOptimisticPieces] = useState<RenderedPiece[]>([]); 

  // Sync optimistic pieces whenever the backend board prop updates
  useEffect(() => {
    if (!board) return;
    setSelectedMoves(null);
    setSelectedSquare(null);
    setOptimisticPieces(prev => {
      const nextPiecesMap = new Map<number, RenderedPiece>();

      prev.forEach(p => {
        nextPiecesMap.set(p.piece.id, { ...p, to_render: false });
      });

      board.squares.forEach((row, rIdx) => {
        row.forEach((p, cIdx) => {
          if (p) {
            const existing = nextPiecesMap.get(p.id);
            if (existing) {
              nextPiecesMap.set(p.id, {
                ...existing,
                piece: p,
                r: rIdx,
                c: cIdx,
                to_render: true
              });
            } else {
              nextPiecesMap.set(p.id, {
                piece: p,
                r: rIdx,
                c: cIdx,
                to_render: true
              });
            }
          }
        });
      });

      return Array.from(nextPiecesMap.values());
    });
  }, [board]);

  // Sync last move highlight from parent
  useEffect(() => {
    if (!lastMove) {
      setLastMoveMade(null);
      return;
    }
    setLastMoveMade(lastMove);
  }, [lastMove]);

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
  }, []);

  const isLegalMove = useCallback((r: number, c: number): boolean => {
    if (!selectedMoves) return false;
    return (
      selectedMoves.capture_moves.some((m) => m[0] === r && m[1] === c) ||
      selectedMoves.quiet_moves.some((m) => m[0] === r && m[1] === c)
    );
  }, [selectedMoves]);

  const selectSquare = useCallback((r: number, c: number) => {
    const piece = board.squares[r][c];
    if (!piece) return;
    if (user ? piece.color !== user : false) return;
    setSelectedSquare([r, c]);
    setSelectedMoves(board.move_cache[piece.id] || null);
  }, [board, user]);

  const deselectSquare = useCallback(() => {
    setSelectedSquare(null);
    setSelectedMoves(null);
  }, []);

  const handlePromotionSelect = useCallback((pieceType: PieceType) => {
    if (!promotionMove) return;
    const { from, to } = promotionMove;

    setOptimisticPieces(prev => {
      return prev.map(p => {
        if (p.r === to[0] && p.c === to[1] && p.to_render) {
          return { ...p, to_render: false };
        }
        if (p.r === from[0] && p.c === from[1] && p.to_render) {
          return {
            ...p,
            r: to[0],
            c: to[1],
            piece: { ...p.piece, kind: pieceType }
          };
        }
        return p;
      });
    });

    setSelectedSquare(null);
    setSelectedMoves(null);
    setPromotionMove(null);

    if (onMove) {
      onMove(from, to, pieceType);
    }
  }, [promotionMove, onMove]);

  const onSquareMouseDown = useCallback((r: number, c: number, e: React.MouseEvent) => {
    if (e.button === 0) {
      setArrows([]);

      if (selectedSquare) {
        const [fromR, fromC] = selectedSquare;

        if (fromR === r && fromC === c) {
          deselectSquare();
          return;
        }

        if (isLegalMove(r, c)) {
          const movingPiece = board.squares[fromR][fromC];
          if (movingPiece && movingPiece.kind === "Pawn") {
            const isWhitePromotion = movingPiece.color === "White" && r === 0;
            const isBlackPromotion = movingPiece.color === "Black" && r === 7;

            if (isWhitePromotion || isBlackPromotion) {
              setPromotionMove({
                from: [fromR, fromC],
                to: [r, c],
                color: movingPiece.color
              });
              return;
            }
          }

          // Optimistic update
          setOptimisticPieces(prev => {
            return prev.map(p => {
              if (p.r === r && p.c === c && p.to_render) {
                return { ...p, to_render: false };
              }
              if (p.r === fromR && p.c === fromC && p.to_render) {
                return { ...p, r: r, c: c };
              }
              return p;
            });
          });

          deselectSquare();
          setLastMoveMade([[fromR, fromC], [r, c]]);
          if (onMove) {
            onMove([fromR, fromC], [r, c]);
          }
          return;
        }
      }

      // Normal selection logic
      const piece = board.squares[r][c];
      const piece_color = piece?.color;

      if (!piece) {
        deselectSquare();
        return;
      }

      if (piece_color !== board.turn) {
        return;
      }
      selectSquare(r, c);
    }
    if (e.button === 2) {
      setStartArrow([r, c]);
    }
  }, [board, selectedSquare, isLegalMove, onMove, deselectSquare, selectSquare]);

  const onSquareMouseUp = useCallback((r: number, c: number, e: React.MouseEvent) => {
    if (e.button === 2) {
      if (startArrow) {
        if (startArrow[0] !== r || startArrow[1] !== c) {
          setArrows(prev => [
            ...prev,
            { from: [startArrow[0], startArrow[1]], to: [r, c], color: "orange", type: "user" as const },
          ]);
        } else {
          setArrows([]);
        }
      }
      setStartArrow(null);
    }
  }, [startArrow]);

  return {
    selectedSquare,
    selectedMoves,
    optimisticPieces,
    arrows,
    lastMoveMade,
    promotionMove,
    onSquareMouseDown,
    onSquareMouseUp,
    handlePromotionSelect,
    handleContextMenu,
    isLegalMove,
  };
}
