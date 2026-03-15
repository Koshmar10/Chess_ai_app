import React, { useState } from 'react';

import { Check, ChevronDown, Icon, RefreshCcw, Search, Upload } from 'lucide-react';
import { IconButton } from '../ui/IconButton';

type SelectOption = "Last Month" | "Last 3 Months" | "All Time";
interface Props {
  onSyncClick: () => Promise<void>;
  onLoadClick: () => void;
  setQueryString: React.Dispatch<React.SetStateAction<string | null>>;
  queryString: string | null;
  setIsLoading: React.Dispatch<React.SetStateAction<boolean>>;
  setDateConstraint: React.Dispatch<React.SetStateAction<string | null>>;
  syncingGames: boolean;
}

const SearchInputs = ({ onSyncClick, onLoadClick, queryString: _queryString, setQueryString, setIsLoading, setDateConstraint, syncingGames }: Props) => {
  const [monthSelectOpen, setMonthSelectOpen] = useState<boolean>(false);
  const [selectOption, setSelectOption] = useState<SelectOption>("Last Month");
  const [localQueryString, setLocalQueryString] = useState<string | null>(null);

  const getDateConstraintForOption = (option: SelectOption): string | null => {
    const now = new Date();
    switch (option) {
      case "Last Month": {
        // Explicitly subtract 1 month
        const d = new Date(now);
        d.setMonth(d.getMonth() - 1);
        return d.toISOString().split('T')[0];
      }
      case "Last 3 Months": {
        // Explicitly subtract 3 months
        // JS Date automatically handles year rollover (e.g., Jan index 0 - 3 = -3 => Oct of previous year)
        const d = new Date(now);
        d.setMonth(d.getMonth() - 3);
        return d.toISOString().split('T')[0];
      }
      case "All Time":
        return null
      default:
        return null;
    }
  };
  const handleOptionClick = (option: SelectOption) => {
    setSelectOption(option);

    // 1. Trigger loading
    setIsLoading(true);

    // 2. Set date constraint
    const constraint = getDateConstraintForOption(option);
    setDateConstraint(constraint);

    // 3. Delay closing to show interaction and allow state propagation
    setTimeout(() => {
      setMonthSelectOpen(false);
      setIsLoading(false);
    }, 500);
  };

  const handleSearchSubmit = () => {
    // 1. Trigger loading state
    setIsLoading(true);
    // 2. Set the global query string to trigger filter
    setQueryString(localQueryString);

    // 3. Set a timeout to turn off loading, simulating data processing and ensuring the spinner is visible
    setTimeout(() => {
      setIsLoading(false);
    }, 500);
  };

  return (
    <div className="gap-4 text-foreground-dark border-b-2 border-border-dark flex flex-row items-center px-4 py-4">
      <div className="relative flex items-center bg-input-dark/30 border-2 border-border-dark rounded px-3 py-2 mr-4 outline-none focus-within:border-primary-dark transition-colors">
        <Search className="text-muted-foreground-dark mr-2 w-4 h-4" />
        <input
          type="text"
          placeholder="Search games..."
          className="bg-transparent outline-none text-foreground-dark placeholder:text-muted-foreground w-full"
          value={localQueryString || ""}
          onChange={(e) => setLocalQueryString(e.target.value)}
          onBlur={handleSearchSubmit}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              handleSearchSubmit();
            }
          }}
        />
      </div>
      <div
        className="w-[11rem] relative bg-card-dark/30 border-2 border-border-dark rounded cursor-pointer select-none flex flex-row items-center justify-between px-2 py-2"
        onClick={() => setMonthSelectOpen((open) => !open)}
      >
        <span className="block">{selectOption}</span>
        <ChevronDown />
        {monthSelectOpen && (
          <div
            className={`
                    absolute left-0 top-full mt-2 bg-card-dark border-2 border-border-dark rounded shadow-lg z-10 min-w-max
                    transition-all duration-200 ease-out
                    opacity-0 scale-95
                    ${monthSelectOpen ? "opacity-100 scale-100" : ""}
                `}
          >
            {(["Last Month", "Last 3 Months", "All Time"] as SelectOption[]).map(option => (
              <div
                key={option}
                className="px-4 py-2 hover:bg-border-dark cursor-pointer flex flex-row w-[12rem] justify-between"
                onClick={() => handleOptionClick(option)}
              >
                <span className="flex-1">{option}</span>
                <span>{option === selectOption ? <Check /> : ''}</span>
              </div>
            ))}
          </div>
        )}
      </div>
      <IconButton
        icon={<RefreshCcw className={`${syncingGames ? 'animate-spin' : ''}`} />}
        tooltip={"Sync with chess.com"}
        onClick={onSyncClick}
        disabled={syncingGames}
      />
      <IconButton icon={<Upload />} tooltip={"Import PGN"} onClick={onLoadClick} />
    </div>
  );
};

export default SearchInputs;