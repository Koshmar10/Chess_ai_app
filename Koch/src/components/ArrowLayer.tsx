import { ChessArrow } from "./ChessArrow";
import { ArrowData } from "./chessboard/types";

// Re-export ArrowData from the shared types
export type { ArrowData } from "./chessboard/types";

export type SquarePosition = {
    row: number;
    col: number;
}

interface ArrowLayerProps {
    arrows: ArrowData[];
    suggestion: ArrowData | null;
    threat: ArrowData | null;
    isFlipped?: boolean;
    squareSize: number;
    ghostArrows?: ArrowData[];
}

export function ArrowLayer({ arrows, suggestion, threat, ghostArrows, isFlipped, squareSize }: ArrowLayerProps) {
    return (
        <svg className="absolute top-0 left-0 w-full h-full z-40 pointer-events-none">
            {arrows.map((arrow, index) => (
                <ChessArrow
                    key={`${arrow.from[0]}-${arrow.from[1]}-${arrow.to[0]}-${arrow.to[1]}-${index}`}
                    from={arrow.from}
                    to={arrow.to}
                    color={arrow.color}
                    isFlipped={isFlipped}
                    squareSize={squareSize}
                />
            ))}

            {suggestion && (
                <ChessArrow
                    key={`suggestion-${suggestion.from[0]}-${suggestion.from[1]}-${suggestion.to[0]}-${suggestion.to[1]}`}
                    from={suggestion.from}
                    to={suggestion.to}
                    color={suggestion.color ?? "cyan"}
                    isFlipped={isFlipped}
                    squareSize={squareSize}
                />
            )}
            {threat && (
                <ChessArrow
                    key={`threat-${threat.from[0]}-${threat.from[1]}-${threat.to[0]}-${threat.to[1]}`}
                    from={threat.from}
                    to={threat.to}
                    color={threat.color ?? "cyan"}
                    isFlipped={isFlipped}
                    squareSize={squareSize}
                />
            )}
            {ghostArrows &&
                ghostArrows.map((arrow, index) => (
                    <ChessArrow
                        key={`ghost-${arrow.from[0]}-${arrow.from[1]}-${arrow.to[0]}-${arrow.to[1]}-${index}`}
                        isGhost={true}
                        from={arrow.from}
                        to={arrow.to}
                        color={arrow.color}
                        isFlipped={isFlipped}
                        squareSize={squareSize}
                    />
                ))
            }
        </svg>
    );
}

export function squareFromCoords(absX: number, absY: number, squareSize: number): number[] {
    const layer = document.getElementById("arrow-layer") || document.querySelector(".relative.inline-block.select-none") as HTMLElement;
    const bounds = layer?.getBoundingClientRect();
    if (!bounds) return [0, 0];
    const relativeX = Math.abs(absX - bounds.x);
    const relativeY = Math.abs(absY - bounds.y);
    const col = Math.floor(relativeX / squareSize);
    const row = Math.floor(relativeY / squareSize);
    return [row, col];
}
