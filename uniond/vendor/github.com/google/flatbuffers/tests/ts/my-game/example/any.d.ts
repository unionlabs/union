import { Monster as MyGame_Example2_Monster } from '../../my-game/example2/monster.js';
import { Monster } from '../../my-game/example/monster.js';
import { TestSimpleTableWithEnum } from '../../my-game/example/test-simple-table-with-enum.js';
export declare enum Any {
    NONE = 0,
    Monster = 1,
    TestSimpleTableWithEnum = 2,
    MyGame_Example2_Monster = 3
}
export declare function unionToAny(type: Any, accessor: (obj: Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum) => Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum | null): Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum | null;
export declare function unionListToAny(type: Any, accessor: (index: number, obj: Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum) => Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum | null, index: number): Monster | MyGame_Example2_Monster | TestSimpleTableWithEnum | null;
