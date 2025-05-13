import { invoke } from "@tauri-apps/api/core";
import { WindowInfo } from "../App";

interface GroupedWindows {
  [key: number]: WindowInfo[];
}

export default function LiveWindowCarousel({
  live_windows,
}: {
  live_windows: WindowInfo[];
}) {
  if (!live_windows || live_windows.length === 0) {
    return <div>No live windows</div>;
  }
  const groupedWindows = live_windows.reduce(
    (acc: GroupedWindows, window: WindowInfo) => {
      const { desktop_index } = window;
      if (!acc[desktop_index]) {
        acc[desktop_index] = [];
      }
      acc[desktop_index].push(window);
      return acc;
    },
    {} as GroupedWindows
  );
  console.log("groupedWindows", groupedWindows[0]);
  return (
    <div className="flex flex-col gap-2">
      {Object.entries(groupedWindows).map(([desktopIndex, windows]) => (
        <div key={desktopIndex} className="flex flex-col gap-2">
          <h2 className="text-xl font-bold">Desktop {desktopIndex}</h2>
          
          <div className="flex flex-row flex-wrap gap-1">
            {windows.map((window: WindowInfo, index: number) => (
              <div
                key={`${window.process_id}-${index}`}
                className="border-2 p-2 rounded-lg"
              >
                <h3 className="text-lg font-semibold">{window.title}</h3>
              </div>
            ))}
            {/* {windows} */}
          </div>
        </div>
      ))}
    </div>
  );
}
