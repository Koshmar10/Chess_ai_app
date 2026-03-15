import React from 'react';

interface BoardSquareProps {
  size: number;
  dark: boolean;
  selected: boolean;
  quietMove: boolean;
  attackMove: boolean;
  captureMove: boolean;
  inCheck: boolean;
  tint: string | undefined;
  hover?: boolean;
  puzzleMoveCorrect?: boolean | null;
  handleMouseDown: (e: React.MouseEvent<HTMLDivElement>) => void;
  handleMouseUp: (e: React.MouseEvent<HTMLDivElement>) => void;
}

export const BoardSquare = React.memo(function BoardSquare({
  size,
  dark,
  inCheck,
  selected,
  quietMove,
  captureMove,
  attackMove,
  tint,
  handleMouseDown,
  handleMouseUp,
  hover = false,
  puzzleMoveCorrect = null,
}: BoardSquareProps) {
  const selectedColor = puzzleMoveCorrect !== null ? (puzzleMoveCorrect ? "bg-[#598c00ff]" : "bg-[#ba5541ff]") : "bg-[#f6f669]";

  const baseStyle: React.CSSProperties = {
    width: size,
    height: size,
  };

  const baseBgClass = attackMove
    ? "bg-red-500"
    : inCheck
      ? "bg-red-200"
      : selected
        ? selectedColor
        : dark
          ? "bg-[#a37a58]"
          : "bg-[#f0d9b5]";

  const highlightClass = hover ? "bg-lime-600/90" : "";
  const combinedBgClass = [baseBgClass, highlightClass].filter(Boolean).join(" ");

  return (
    <div
      className={[
        "relative",
        combinedBgClass,
      ].join(" ")}
      style={baseStyle}
      onMouseDown={(e) => handleMouseDown(e)}
      onMouseUp={(e) => handleMouseUp(e)}
      onContextMenu={(e) => e.preventDefault()}
    >
      {tint && (
        <div
          className="absolute inset-0 pointer-events-none rounded-sm"
          style={{
            backgroundColor: tint,
            zIndex: 10,
          }}
        />
      )}

      {quietMove && (
        <div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-gray-500 opacity-85 z-30"
          style={{
            width: Math.max(8, Math.floor(size * 0.3)),
            height: Math.max(8, Math.floor(size * 0.3)),
          }}
        />
      )}
      {captureMove && (
        <div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full border-[3px] border-gray-500/90 box-border z-30"
          style={{
            width: Math.max(12, Math.floor(size * 0.6)),
            height: Math.max(12, Math.floor(size * 0.6)),
          }}
        />
      )}
    </div>
  );
});
