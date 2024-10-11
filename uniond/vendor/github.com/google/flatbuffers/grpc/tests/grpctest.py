from __future__ import print_function

import os
import sys
import grpc
import flatbuffers

from concurrent import futures

sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..', 'tests'))
import MyGame.Example.Monster as Monster
import MyGame.Example.Stat as Stat
import MyGame.Example.Vec3 as Vec3
import MyGame.Example.Test as Test
import MyGame.Example.monster_test_grpc_fb as monster_grpc_fb


test_stat_id = "test_stat_id"
test_stat_val = 8
test_stat_count = 1

test_monster_name1 = "test_monster_name1"
test_monster_name2 = "test_monster_name2"
test_string = "test_string"
test_color = 2
test_X = 3.0
test_Y = 2.0
test_Z = 6.0
test_test1 = 4.0
test_a = 8
test_b = 5
test_hp = 67
test_inventory = [1, 1, 2, 3, 5, 8]
test_testtype = 4

test_monsters_name_retrieve = ["big_monster", "small_monster"]
test_no_of_monsters = 2


class MonsterStorage(monster_grpc_fb.MonsterStorageServicer):

    def Store(self, request, context):

        m = Monster.Monster().GetRootAsMonster(request, 0)

        assert m.Name().decode("utf-8") == test_monster_name1

        assert m.Pos().X() == test_X
        assert m.Pos().Y() == test_Y
        assert m.Pos().Z() == test_Z
        assert m.Pos().Test1() == test_test1
        assert m.Pos().Test2() == test_color
        test3 = Test.Test()
        assert m.Pos().Test3(test3).A() == test_a
        assert m.Pos().Test3(test3).B() == test_b

        assert m.Hp() == test_hp

        assert m.Color() == test_color

        assert m.InventoryLength() == len(test_inventory)
        for i in range(0, len(test_inventory)):
            assert m.Inventory(i) == test_inventory[len(test_inventory)-i -1]

        assert m.TestType() == test_testtype

        assert m.Test() is not None
        table = m.Test()

        m2 = Monster.Monster()
        m2.Init(table.Bytes, table.Pos)
        assert m2.Name().decode("utf-8") == test_monster_name2

        m3 = m.Enemy()
        assert m3.Name().decode("utf-8") == test_monster_name2

        assert m.Testarrayofstring(0).decode("utf-8") == test_string

        b = flatbuffers.Builder(0)
        i = b.CreateString(test_stat_id)
        Stat.StatStart(b)
        Stat.StatAddId(b, i)
        Stat.StatAddVal(b, test_stat_val)
        Stat.StatAddCount(b, test_stat_count)
        b.Finish(Stat.StatEnd(b))
        return bytes(b.Output())

    def Retrieve(self, request, context):

        s = Stat.Stat().GetRootAsStat(request, 0)

        no_of_monsters = test_no_of_monsters
        for i in range(0, no_of_monsters):
            b = flatbuffers.Builder(0)
            i = b.CreateString(test_monsters_name_retrieve[i])
            Monster.MonsterStart(b)
            Monster.MonsterAddName(b, i)
            b.Finish(Monster.MonsterEnd(b))
            yield bytes(b.Output())


def serve():

    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    monster_grpc_fb.add_MonsterStorageServicer_to_server(MonsterStorage(), server)
    server.add_insecure_port('[::]:50051')

    server.start()

    run()


def run():

    channel = grpc.insecure_channel('127.0.0.1:50051')
    stub = monster_grpc_fb.MonsterStorageStub(channel)

    b = flatbuffers.Builder(0)
    name2 = b.CreateString(test_monster_name2)
    name1 = b.CreateString(test_monster_name1)
    Monster.MonsterStart(b)
    Monster.MonsterAddName(b, name2)
    monster2 = Monster.MonsterEnd(b)
    test1 = b.CreateString(test_string)

    Monster.MonsterStartInventoryVector(b, len(test_inventory))
    for i in range(0, len(test_inventory)):
        b.PrependByte(test_inventory[i])
    inv = b.EndVector()

    Monster.MonsterStartTest4Vector(b, 2)
    Test.CreateTest(b, 10, 20)
    Test.CreateTest(b, 30, 40)
    test4 = b.EndVector()

    Monster.MonsterStartTestarrayofstringVector(b, 1)
    b.PrependUOffsetTRelative(test1)
    test_array_of_string = b.EndVector()

    Monster.MonsterStart(b)

    Monster.MonsterAddHp(b, test_hp)
    Monster.MonsterAddName(b, name1)
    Monster.MonsterAddColor(b, test_color)
    pos = Vec3.CreateVec3(b, test_X, test_Y, test_Z, test_test1, test_color, test_a, test_b)
    Monster.MonsterAddPos(b, pos)
    Monster.MonsterAddInventory(b, inv)
    Monster.MonsterAddTestType(b, test_testtype)
    Monster.MonsterAddTest(b, monster2)
    Monster.MonsterAddTest4(b, test4)
    Monster.MonsterAddEnemy(b, monster2)
    Monster.MonsterAddTestarrayofstring(b, test_array_of_string)
    monster = Monster.MonsterEnd(b)

    b.Finish(monster)

    stat_response = stub.Store(bytes(b.Output()))

    s = Stat.Stat().GetRootAsStat(stat_response, 0)

    assert s.Id().decode("utf-8") == test_stat_id
    assert s.Val() == test_stat_val
    assert s.Count() == test_stat_count

    monster_reponses = stub.Retrieve(stat_response)
    count = 0
    for monster_reponse in monster_reponses:
        m = Monster.Monster().GetRootAsMonster(monster_reponse, 0)
        assert m.Name().decode("utf-8") == test_monsters_name_retrieve[count]
        count = count + 1


if __name__ == '__main__':
    serve()
