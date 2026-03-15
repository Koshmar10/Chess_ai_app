import { PieceMoves } from "../../../src-tauri/bindings/PieceMoves"
export type MoveChache = {
    [x: number]: PieceMoves | undefined;
}

