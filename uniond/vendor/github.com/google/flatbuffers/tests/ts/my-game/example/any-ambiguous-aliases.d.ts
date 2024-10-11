import { Monster } from '../../my-game/example/monster.js';
export declare enum AnyAmbiguousAliases {
    NONE = 0,
    M1 = 1,
    M2 = 2,
    M3 = 3
}
export declare function unionToAnyAmbiguousAliases(type: AnyAmbiguousAliases, accessor: (obj: Monster) => Monster | null): Monster | null;
export declare function unionListToAnyAmbiguousAliases(type: AnyAmbiguousAliases, accessor: (index: number, obj: Monster) => Monster | null, index: number): Monster | null;
