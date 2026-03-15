import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { BoardMetaData } from "../../../src-tauri/bindings/BoardMetaData";
import { LoadPgnPopup } from "./LoadPgnPopup";
import { Settings } from "../../../src-tauri/bindings/Settings";
import { GameCard } from "./GameCard"; // Import the new GameCard component
import SearchInputs from "./SearchInputs";
import ActivityChart from "./ActivityChart";
// Add Loader2 icon import
import { Loader2 } from "lucide-react";

interface HistoryProps {
  onOpenGame?: (id: number) => void;
}

export function History({ onOpenGame }: HistoryProps) {
  const [pastGames, setPastGames] = useState<[number, BoardMetaData][]>([]);
  const [loadPgnPopupOpen, setLoadPgnPopupOpen] = useState<boolean>(false);
  const [reloadKey, setReloadKey] = useState<number>(0);
  const [chessdotcomSyncAllowed, setChessdotcomSyncAllowed] = useState<boolean>(false);
  const [queryString, setQueryString] = useState<string | null>(null);
  const [dateConstraint, setDateConstraint] = useState<string | null>(null);
  // 1. Add loading state
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const [syncingGames, setSyncingGames] = useState<boolean>(false)
  const filter_game = (game: BoardMetaData) => {
    // 1. Text Filter
    const q = queryString?.toLowerCase() || "";
    const matchesText = !q || (
      game.white_player_name?.toLowerCase().includes(q) ||
      game.black_player_name?.toLowerCase().includes(q) ||
      game.opening?.toLowerCase().includes(q) ||
      game.event?.toLowerCase().includes(q) ||
      game.site?.toLowerCase().includes(q) ||
      game.round?.toLowerCase().includes(q) ||
      game.time_control?.toLowerCase().includes(q) ||
      game.eco?.toLowerCase().includes(q) ||
      game.result?.toLowerCase().includes(q) ||
      game.termination?.toLowerCase().includes(q) ||
      game.date?.toLowerCase().includes(q) ||
      false
    );

    // 2. Date Filter
    // Replace dots with dashes to ensure standard parsing (PGN usually has YYYY.MM.DD)
    const normalizedDateStr = game.date.replace(/\./g, '-');
    const gameDate = new Date(normalizedDateStr);
    // Ensure we clear time part to compare dates only
    gameDate.setHours(0, 0, 0, 0);

    let matchesDate = true;

    if (dateConstraint) {
      const constraintDate = new Date(dateConstraint);
      constraintDate.setHours(0, 0, 0, 0);

      // If invalid date (e.g. game date missing), decide whether to show or hide.
      // Usually valid game dates behave like numbers, invalid ones behave like NaN.
      if (isNaN(+gameDate)) {
        // If gameDate is invalid, we might want to hide such games
        matchesDate = false;
      } else {
        matchesDate = gameDate >= constraintDate;
      }
    }

    return matchesText && matchesDate;
  }
  const handleDeleteGame = async (id: number) => {
    const res = await invoke<string | null>('delete_game', { id: id });
    if (res == null) {
      setPastGames((prev) => prev.filter((game) => game[0] !== id));
    }
  }
  useEffect(() => {
    async function func() {
      // 2. Set loading to true before fetch
      setIsLoading(true);
      try {
        const sync = await invoke<boolean>("is_syncing_chessdotcom");
        setSyncingGames(sync);
        const games = await invoke<[number, BoardMetaData][]>("fetch_game_history");
        setPastGames(games);
      } catch (error) {
        console.error("Failed to fetch history:", error);
      } finally {
        // 3. Set loading to false after fetch completes
        setIsLoading(false);
      }
    }
    func();
  }, [reloadKey]);

  useEffect(() => {
    const get_user = async () => {
      try {
        const s = await invoke<Settings>("get_settings");
        const hasUser = Boolean(s.map && s.map["chessdotcom_user"]);
        setChessdotcomSyncAllowed(hasUser);
      } catch (err) {
        console.error("get_settings failed:", err);
        setChessdotcomSyncAllowed(false);
      }
    };
    get_user();
  }, []);
  const handleChessdotcomSync = async () => {
    if (!chessdotcomSyncAllowed) return;
    if (syncingGames) return;
    setChessdotcomSyncAllowed(false);
    setSyncingGames(true);
    try {
      await invoke<null>("sync_with_chessdotcom");
      setReloadKey((k) => k + 1);
    } catch (err) {
      console.error("sync_with_chessdotcom failed:", err);
    } finally {
      setChessdotcomSyncAllowed(true);
      setSyncingGames(false);
    }
  };
  useEffect(() => {
    let intervalId: ReturnType<typeof setInterval>;

    if (syncingGames) {
      intervalId = setInterval(async () => {
        try {
          const games = await invoke<[number, BoardMetaData][]>("fetch_game_history");
          setPastGames(games);

          // Check if sync is still running
          const isSyncing = await invoke<boolean>("is_syncing_chessdotcom");
          if (!isSyncing) {
            setSyncingGames(false);
          }
        } catch (e) {
          console.error("Error polling games during sync:", e);
        }
      }, 1000);
    }

    return () => {
      if (intervalId) clearInterval(intervalId);
    };
  }, [syncingGames]);
  return (
    <div className="flex flex-col w-[100%] justify-start">
      <LoadPgnPopup
        popupOpen={loadPgnPopupOpen}
        setPopupOpen={setLoadPgnPopupOpen}
        onLoadSuccess={() => setReloadKey((k) => k + 1)}
      />
      <div className="bg-card-dark/20 border-b-2 border-border-dark w-full flex flex-start">
        <h1 className="pl-2 pt-4 pb-4 text-xl font-normal text-foreground-dark">Game History</h1>
      </div>
      <ActivityChart />
      <SearchInputs onSyncClick={handleChessdotcomSync} onLoadClick={() => { setLoadPgnPopupOpen(true) }}
        queryString={queryString} setQueryString={setQueryString} setIsLoading={setIsLoading} setDateConstraint={setDateConstraint} syncingGames={syncingGames} />

      {/* 4. Display Spinner or Content based on isLoading */}
      <div className="w-full flex flex-row flex-wrap gap-4 p-4 overflow-y-scroll min-h-[200px]">
        {isLoading ? (
          <div className="w-full h-full flex justify-center items-center py-10">
            <Loader2 className="animate-spin text-primary-dark w-10 h-10" />
          </div>
        ) : (
          <>
            {pastGames.length === 0 && (
              <div className="w-full text-center text-muted-dark py-10">No games found.</div>
            )}
            {pastGames.map((pg, idx) => (
              <GameCard
                key={idx + 1}
                render={filter_game(pg[1])}
                game={pg[1]}
                onClick={() => onOpenGame && onOpenGame(pg[0])}
                onDeleteGame={handleDeleteGame}
                game_id={pg[0]}
              />

            ))}
          </>
        )}
      </div>
    </div>
  );
}