// Puzzle example 
// {
//   "game": {
//     "id": "LMXI7KPi",
//     "perf": {
//       "key": "rapid",
//       "name": "Rapid"
//     },
//     "rated": true,
//     "players": [
//       {
//         "name": "vahid96",
//         "id": "vahid96",
//         "color": "white",
//         "rating": 1850
//       },
//       {
//         "name": "radmehr1358",
//         "id": "radmehr1358",
//         "color": "black",
//         "rating": 1833
//       }
//     ],
//     "pgn": "Nf3 d5 g3 c5 Bg2 Nc6 d3 e5 O-O Nf6 e4 d4 a4 b6 Bd2 Bg4 a5 Bd6 h3 Bh5 Na3 bxa5 Nc4 Rb8 Bxa5 Nxa5 Rxa5 Bxf3 Bxf3 Rb7 Qa1 Bb8 Rxc5 Nd7 Ra5 O-O Qa2 Kh8 Bg4 f6 Bxd7 Qxd7 Kg2 f5 Nxe5 Bxe5 Rxe5 fxe4 Rxe4 Qc6 b3 Rc7 Rc1 Qf6 Rf4 Qc6+ Kf1",
//     "clock": "5+10"
//   },
//   "puzzle": {
//     "id": "BLbOg",
//     "rating": 1953,
//     "plays": 18844,
//     "solution": [
//       "f8f4",
//       "g3f4",
//       "c6h1"
//     ],
//     "themes": [
//       "endgame",
//       "short",
//       "crushing"
//     ],
//     "initialPly": 56
//   }
// }
import { invoke } from "@tauri-apps/api/core";
import { Flame, RefreshCwIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { ChessBoard } from "../chessboard/Chessboard";
import type { LichessPuzzleResponse } from "../../../src-tauri/bindings/LichessPuzzleResponse";
import { Board } from "../../../src-tauri/bindings/Board";

export function PuzzlePanel() {
  const [puzzleData, setPuzzleData] = useState<LichessPuzzleResponse | null>(null);
  const [parseError, setParseError] = useState<string | null>(null);
  const [board, setBoard] = useState<Board | null>(null);
  const [puzzleMoveIndex, _setPuzzleMoveIndex] = useState<number>(0);
  const [puzzleMoveResult, setPuzzleMoveResult] = useState<boolean | null>(null);
  const getPuzzle = async (flagg: string) => {
    try {
      const puzzle = await invoke<[Board, LichessPuzzleResponse]>("get_puzzle", {
        flag: flagg
      });
      setPuzzleData(puzzle[1]);
      setBoard(puzzle[0])
      setParseError(null);
    } catch (e) {
      setPuzzleData(null);
      setParseError(e instanceof Error ? e.message : String(e));
    }
  };
  useEffect(() => {
    getPuzzle("OnLoad");
  }, []);
  const coordsToSquare = (from: [number, number], to: [number, number]) => {
    const files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    let sq1 = `${files[from[1]]}${8 - from[0]}`;
    let sq2 = `${files[to[1]]}${8 - to[0]}`;
    return sq1 + sq2;
  }
  const checkSolutionMatch = (madeMove: string) => {
    if (!puzzleData) return false;
    return madeMove === puzzleData.puzzle.solution[puzzleMoveIndex];
  }
  const handleMove = async (from: [number, number], to: [number, number], _promotion?: string) => {
    const uciMove = coordsToSquare(from, to);
    const moveRez = checkSolutionMatch(uciMove);
    setPuzzleMoveResult(moveRez);
  }
  return (
    <div className="text-foreground-dark flex flex-col gap-12">
      <span>Puzzle Section</span>

      <button onClick={() => { getPuzzle("GetNew") }}>
        <RefreshCwIcon />
      </button>

      {parseError ? (
        <span className="text-red-500">Failed to parse puzzle JSON: {parseError}</span>
      ) : (
        <div className="flex flex-row h-full gap-4 justify-start items-start w-[80%] mx-auto">
          <div className="flex flex-col gap-8 w-[60%]">

            <div className="flex flex-col justify-center">
              <span>4,567</span>

              <div className="flex flex-row items-center gap-4">
                <div className="relative w-60">
                  <div className="absolute top-0 left-0 w-[40%] h-4 bg-yellow-400 rounded-full" />
                  <div className="absoulute top-0 left-0 w-[100%] h-4 bg-card-dark rounded-full" />
                </div>
                <div className="flex flex-row items-center ">
                  <Flame />
                  5
                </div>
              </div>
            </div>
            {puzzleData?.game ? JSON.stringify(puzzleData.game, null, 2) : "No puzzle loaded"}
            {board && <ChessBoard board={board} squareSize={70} onMove={handleMove} wasCorrectPuzzleMove={puzzleMoveResult} />}
          </div>

          <div className="puzzle-info bg-card-dark/80 p-4 w-[40%] flex flex-col gap-2">
            <div className="flex flex-row justify-between">

              {puzzleData && puzzleData.puzzle.themes && (
                <div className="flex flex-wrap gap-2">
                  {puzzleData.puzzle.themes.map((theme, idx) => (
                    <div key={idx} className="badge bg-primary/20 p-1 rounded text-sm">
                      {theme}
                    </div>
                  ))}
                </div>
              )}
              {puzzleData &&
                <div>
                  Rating: {puzzleData.puzzle.rating}
                </div>
              }
            </div>

            {puzzleData?.puzzle ? JSON.stringify(puzzleData.puzzle, null, 2) : "No puzzle loaded"}
            <div className="flex flex-row flex-wrap gap-2 max-w-[40rem]">
              {board
                ? board.meta_data.move_list.map((mv, idx) => (
                  <div
                    key={idx}
                    className=""
                  >
                    {mv.san}
                  </div>
                ))
                : null}
            </div>
          </div>

        </div>
      )}
    </div>
  );
}
