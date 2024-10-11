import { Attacker } from './attacker.js';
import { BookReader } from './book-reader.js';
import { Rapunzel } from './rapunzel.js';
export declare enum Character {
    NONE = 0,
    MuLan = 1,
    Rapunzel = 2,
    Belle = 3,
    BookFan = 4,
    Other = 5,
    Unused = 6
}
export declare function unionToCharacter(type: Character, accessor: (obj: Attacker | BookReader | Rapunzel | string) => Attacker | BookReader | Rapunzel | string | null): Attacker | BookReader | Rapunzel | string | null;
export declare function unionListToCharacter(type: Character, accessor: (index: number, obj: Attacker | BookReader | Rapunzel | string) => Attacker | BookReader | Rapunzel | string | null, index: number): Attacker | BookReader | Rapunzel | string | null;
