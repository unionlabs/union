import { Monster as MyGame_Example2_Monster } from '../../my-game/example2/monster.js';
import { Monster } from '../../my-game/example/monster.js';
import { TestSimpleTableWithEnum } from '../../my-game/example/test-simple-table-with-enum.js';
export declare enum AnyUniqueAliases {
    NONE = 0,
    M = 1,
    TS = 2,
    M2 = 3
}
export declare function unionToAnyUniqueAliases(type: AnyUniqueAliases, accessor: (obj: Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum) => Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum | null): Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum | null;
export declare function unionListToAnyUniqueAliases(type: AnyUniqueAliases, accessor: (index: number, obj: Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum) => Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum | null, index: number): Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum | null;
