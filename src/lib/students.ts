// The students of SchaleDB's JP data (public/schaledb, tools/schaledb/update.ps1):
// the names a party takes and the faces shown beside them.

export interface SchaleStudent {
  id: number;
  name: string;
  squad: "striker" | "special";
  released: boolean;
}

export type Roster = SchaleStudent[];

/** The face icon of the student named `name`, or null when there is none. */
export function faceOf(roster: Roster, name: string): string | null {
  const wanted = name.trim();
  const student = wanted === "" ? undefined : roster.find((s) => s.name === wanted);
  return student ? `/schaledb/students/${student.id}.webp` : null;
}

/** The names a party group can take. */
export function namesOf(roster: Roster, squad: SchaleStudent["squad"]): string[] {
  return [...new Set(roster.filter((s) => s.squad === squad && s.released).map((s) => s.name))];
}

/** The roster the app was built with; empty when it cannot be read. */
export async function loadRoster(): Promise<Roster> {
  try {
    const response = await fetch("/schaledb/students.json");
    return response.ok ? await response.json() : [];
  } catch {
    return [];
  }
}
