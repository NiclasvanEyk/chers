import { ReactNode, createContext, useContext, useState } from "react";

export const ChersSettingsDefaults = Object.freeze({
  showLegalMoves: true as boolean,
  displayCapturedPieces: true as boolean,
  displayLabels: true as boolean,
  onlyLegalTabTargets: false as boolean,
})

export function ChersSettingsProvider(props: { children: ReactNode }) {
  const settings = useSettingsPrimitive(ChersSettingsDefaults);

  return <ChersSettingsContext.Provider value={settings}>
    {props.children}
  </ChersSettingsContext.Provider>;
}

export function useSettings() {
  return useContext(ChersSettingsContext).all;
}

export function useAdjustableSettings() {
  return useContext(ChersSettingsContext);
}

// ============================================================================
// Everything below this line is a bit over-engineered.
// ============================================================================

type JsonScalarValue = boolean | number | string | null;
type JsonSafeObject = { [key: string]: JsonScalarValue }

interface AdjustmentsStore<T extends JsonSafeObject> {
  save(settings: Partial<T>): void;
  pull(): Partial<T>;
}

interface AdjustableSettings<T> {
  /**
   * The current value of all available settings.
   *
   * This is computed based on the defaults and adjustments made by the user.
   */
  all: T,

  /**
   * Registers an adjusment made by the user.
   *
   * This indicates, that they want to change the current default for a 
   * setting.
   */
  adjust(id: keyof T, value: T[keyof T]): void;

  /**
   * Uses the default value of the setting again.
   */
  reset(id: keyof T): void;
}

function useSettingsPrimitive<T extends JsonSafeObject>(
  defaults: T,
  storeImplementation: AdjustmentsStore<T> | null = null,
): AdjustableSettings<T> {
  const store = storeImplementation ?? new LocalStorageAdjustmentsStore()
  const [adjustments, setAdjustments] = useState(() => store.pull())
  const derived = { ...defaults, ...adjustments };

  return {
    /**
     * The current value of all available settings.
     *
     * This is computed based on the defaults and adjustments made by the user.
     */
    all: derived,

    /**
     * Registers an adjusment made by the user.
     *
     * This indicates, that they want to change the current default for a 
     * setting.
     */
    adjust(id: keyof T, value: T[keyof T]) {
      const newAdjustments = { ...adjustments };
      newAdjustments[id] = value;
      setAdjustments(newAdjustments);
      store.save(newAdjustments)
    },

    /**
     * Uses the default value of the setting again.
     */
    reset(id: keyof T) {
      const newAdjustments = { ...adjustments };
      delete newAdjustments[id];
      setAdjustments(newAdjustments);
    },
  }
}

class LocalStorageAdjustmentsStore<T extends JsonSafeObject> implements AdjustmentsStore<T> {
  save(settings: Partial<T>): void {
    localStorage.setItem("settings", JSON.stringify(settings));
  }

  pull(): Partial<T> {
    const serialized = localStorage.getItem("settings");
    if (!serialized) {
      return {};
    }

    return JSON.parse(serialized);
  }
}

const ChersSettingsContext = createContext<AdjustableSettings<typeof ChersSettingsDefaults>>({
  all: ChersSettingsDefaults,
  reset() { },
  adjust() { },
});

