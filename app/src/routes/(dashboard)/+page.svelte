<script lang="ts">
import {
  flexRender,
  type ColumnDef,
  getCoreRowModel,
  type TableOptions,
  createSvelteTable,
  getFilteredRowModel,
  getPaginationRowModel
} from "@tanstack/svelte-table"
import { Shine } from "svelte-ux"
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { writable } from "svelte/store"
import { CHAIN_MAP } from "$lib/constants/chains"
import * as Table from "$lib/components/ui/table"
import { createQuery } from "@tanstack/svelte-query"
import { removeArrayDuplicates } from "$lib/utilities"
import { rankItem } from "@tanstack/match-sorter-utils"
import type { Override } from "$lib/utilities/types.ts"
import { cn, flyAndScale } from "$lib/utilities/shadcn.ts"
import ChevronLeft from "virtual:icons/lucide/chevron-left"
import { createVirtualizer } from "@tanstack/svelte-virtual"
import * as Select from "$lib/components/ui/select/index.ts"
import Button from "$lib/components/ui/button/button.svelte"
import ChevronRight from "virtual:icons/lucide/chevron-right"
import TimeElapsed from "$lib/components/time-elapsed.svelte"
import DoubleArrowLeft from "virtual:icons/lucide/chevrons-left"
import DoubleArrowRight from "virtual:icons/lucide/chevrons-right"
import { dollarize, relativeTime } from "$lib/utilities/format.ts"
import { cosmosBlocksQuery } from "$lib/graphql/documents/cosmos-blocks.ts"

// $: cosmosBlocks = createQuery({
//   queryKey: ["cosmos-blocks"],
//   refetchInterval: 6_000,
//   // enabled: false,
//   queryFn: async () => request(URLS.GRAPHQL, cosmosBlocksQuery, { limit: 100 })
// })

// $: blockData = $cosmosBlocks?.data?.data ?? []

const blockData = [
  // {
  //   chain_id: 5,
  //   hash: 'B6E24FDF583DA3834C71B0DD120C4A9FF0B801AD934E05E7BD885BB8A9792A32',
  //   height: 10717166,
  //   time: '2024-05-29T07:26:35.092162+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '9F8261EE56F87B27B6D7559E4023F552CED35BC21B8F92A6785D03BF520640B8',
  //   height: 688978,
  //   time: '2024-05-29T07:26:33.002707+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '20C3D408770DDF878120EC706AF868028009ACD403A5C22701D55A0D021430D7',
  //   height: 10717165,
  //   time: '2024-05-29T07:26:29.52839+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '1326A4AEA220CA96B857D45FFDCBF6890BC669924F1939706F823568348CD9C2',
  //   height: 688977,
  //   time: '2024-05-29T07:26:26.989601+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '4DA0FE3EAB11003EE22B97224EE4920B0286F773E694E83E2CDD005D3B07E6CD',
  //   height: 10717164,
  //   time: '2024-05-29T07:26:23.958196+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'A30726168D87A0D902512CBB1A868E8CBFFADBFBA8D8EFB2BFBC8DA621CEDD94',
  //   height: 688976,
  //   time: '2024-05-29T07:26:21.060378+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'AEC27E33D6A99EB323E40ED7B3A6A4DA7DBDDC8DC421D1DEFCAD3850C5F9C834',
  //   height: 10717163,
  //   time: '2024-05-29T07:26:18.379004+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '3546A2AFEE04FCBBAFEEC33C8A258CF715C1EA12BA1626C95C09848D8D92CE5F',
  //   height: 688975,
  //   time: '2024-05-29T07:26:14.837213+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '11D0F798B12B272E5DC341CA01E76002B6094B7904901AB9A596673AF3B18E58',
  //   height: 10717162,
  //   time: '2024-05-29T07:26:12.868362+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '46DB9248205985EBBE64EA407D5E5A03658853651B8C1A603D297BFE7E9CD05D',
  //   height: 688974,
  //   time: '2024-05-29T07:26:08.675793+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '17F874F64BA83C644AC34B333AF2F7544BA59B4531069602028540D96CF45271',
  //   height: 10717161,
  //   time: '2024-05-29T07:26:07.445725+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'DB0818F268A1CE05B6F0492DA25004E48EE350216E81655E8848DB5F83E72077',
  //   height: 688973,
  //   time: '2024-05-29T07:26:02.497203+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '82F6198D9AD3F2C99FAE2044D2702032A677FE5FD0B456B020F5709C02F8FB12',
  //   height: 10717160,
  //   time: '2024-05-29T07:26:01.950307+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'E85BFE3FC9C63DD1E40E9257F0EBA1D10A04FF9402058C0BBE7D42653A29FAFA',
  //   height: 10717159,
  //   time: '2024-05-29T07:25:56.396479+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'AA937A0BD855AE573555E0CC370C9DD30BB419ABD8F256EA46B2234D60B85B05',
  //   height: 688972,
  //   time: '2024-05-29T07:25:55.112566+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'B7C4D45962117C55EEE0826EA45A36A8F48CF501B920D21CB6CE5D6A3BEE01C9',
  //   height: 10717158,
  //   time: '2024-05-29T07:25:50.806727+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '462893D7E38D7ECA129255D5C5486322DC0A649183F347BB6DF2734A4B8FA78C',
  //   height: 688971,
  //   time: '2024-05-29T07:25:48.798065+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '81D031BE34915FC438A8ED81E441E313AFFCDD2131DC3E1791745C63C9A34179',
  //   height: 10717157,
  //   time: '2024-05-29T07:25:45.294447+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'A593E397EF78518DC871EEE2764BCD89C516F7063A273122329CD09380F4F040',
  //   height: 688970,
  //   time: '2024-05-29T07:25:42.701575+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '472A0A2454FA6A1C7D2E940E193FD35DAEEA5A35F2080ED8DAF14F7813FB734E',
  //   height: 10717156,
  //   time: '2024-05-29T07:25:39.789827+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '46DB7F5AEEDD7A878E6D52346A42A78C2166E938C2C6A41C72E3592DA701A118',
  //   height: 688969,
  //   time: '2024-05-29T07:25:36.577533+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '6FCA9E1ECB4DE47A11604E7A5E91C84CA53F7B49F9FC6165A750633085C17AEC',
  //   height: 10717155,
  //   time: '2024-05-29T07:25:34.292787+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'F025424461FBB3E9FE6DB34FBE2192B084E8C6A37ABDD8395801A6CE35C2A392',
  //   height: 688968,
  //   time: '2024-05-29T07:25:30.144715+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '685468C2ED21E78A36BA799ADC5281389E7222A59999151D725877508D188684',
  //   height: 10717154,
  //   time: '2024-05-29T07:25:28.747546+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'B5DA4CE4ECB0456A3FAE9FDFB1A036CDA1C0F08A07603B0C0B75AA7D332C839E',
  //   height: 10717153,
  //   time: '2024-05-29T07:25:23.188173+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '8EF683EF2EB64CA8E5EB76322082007D917F04025537AC110583C241EFFC6E9B',
  //   height: 688967,
  //   time: '2024-05-29T07:25:23.066347+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '055ABAE5DBD8BE5F3080A0950071E4611DF12C63E61288978CA4C0B19172D465',
  //   height: 10717152,
  //   time: '2024-05-29T07:25:17.610188+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'DAB80190E8C0A84C70C3342787BE98D7157FDDBE19175B994E31784897CF5FF8',
  //   height: 688966,
  //   time: '2024-05-29T07:25:16.68867+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'FFE47506AA31B7392DDAD6105998B1B76155FBBFE60768CC89559F61E7C06209',
  //   height: 10717151,
  //   time: '2024-05-29T07:25:12.063463+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '594C46A0FD4154CD9055E85F3535C6CA9DD3712794BB453FF14BD5EFCBF96253',
  //   height: 688965,
  //   time: '2024-05-29T07:25:10.55183+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '0F7142E68BF089940673BA80480B0893DC872FB126DB4DD63862010B02E7F772',
  //   height: 10717150,
  //   time: '2024-05-29T07:25:06.495755+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '3C1CFD1FB99FC1A302C2AF11A30204DF281CB6814A9AB9653DA1D6C234867728',
  //   height: 688964,
  //   time: '2024-05-29T07:25:04.359487+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'D4A9D09D71D80E6F3ED84AA88D3C0531D9C37652B2F43405A6145EDB98AAFE7B',
  //   height: 10717149,
  //   time: '2024-05-29T07:25:00.891781+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'C0100B680A62AAF64E13C2A7226665775C00CD0516AE1B293749F9AC11DA5E30',
  //   height: 688963,
  //   time: '2024-05-29T07:24:58.285092+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '5502A73E39CC5CCD641F1B975E163E6FCDE4B1F911F72726258E13D769A119A4',
  //   height: 10717148,
  //   time: '2024-05-29T07:24:55.273564+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '120E1BD17F6021965DF4BE2530A627B706C08C63A02D1C91F4696D4E3C1104BD',
  //   height: 688962,
  //   time: '2024-05-29T07:24:52.20375+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '595BCE02BE102734039520EE14899A91042F89BADC86B0E394F21FFF6B0FD53B',
  //   height: 10717147,
  //   time: '2024-05-29T07:24:49.734815+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '025981C99A75A974F9ADCF739B10FEB4556B9A05BE5EC1228556B7B565F81507',
  //   height: 688961,
  //   time: '2024-05-29T07:24:45.781989+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '0AC44933D23AAC1C02A4D8BA6F7E6E590A3EBDD93E7F2E468A9C0C935D5362D8',
  //   height: 10717146,
  //   time: '2024-05-29T07:24:44.199626+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'F1666D74F99B154787B32FE69E146122F2D2A80520C6B51CBC2DB234AA68C004',
  //   height: 688960,
  //   time: '2024-05-29T07:24:39.70681+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '66B5460FFE186BC4FAA054B6838EF31485FF395B3A4658B0BD85BA46BFE4A57E',
  //   height: 10717145,
  //   time: '2024-05-29T07:24:38.682051+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'F2CB1104C64B387D6799622666D08E01520C30C670229659837C8D88BEDE7DC1',
  //   height: 688959,
  //   time: '2024-05-29T07:24:33.541798+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '1A4D451E79CDCEC10B68E35E663CCCF8FBC38F92AB249B951466F14C946BF192',
  //   height: 10717144,
  //   time: '2024-05-29T07:24:32.97459+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'C7B19CC600AA89438E2DC33A0F20A7F5821461136A1776C948833D9E6D50701A',
  //   height: 10717143,
  //   time: '2024-05-29T07:24:27.467108+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '4945B199A4060AA1E3CBDF51D6FA953C429482A4367005F673589E3566120845',
  //   height: 688958,
  //   time: '2024-05-29T07:24:27.30996+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'D4206E2D73082EF0F650720C2B34A876372E7B49BA40B081C5BC66975EC301DF',
  //   height: 10717142,
  //   time: '2024-05-29T07:24:22.009892+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '424511DAAAB33BC0708CE8575BC87E7A2DA30274CA3F1F1CF0E9195340ACFD2B',
  //   height: 688957,
  //   time: '2024-05-29T07:24:21.225573+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '8BCF4438FF86145D85C30F336F69E0C5AAECBE7CD27FB5334D9C9F16A1983DC5',
  //   height: 10717141,
  //   time: '2024-05-29T07:24:16.57322+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '3535B639EBE7C6BC19BCCF0060B356BF538F366082E925D6CAAA0FEDFCFEEB0D',
  //   height: 688956,
  //   time: '2024-05-29T07:24:14.785819+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '445172F5765F60AB2D1AA055CBEE067E0BF53A88DF719A405E92872B8390EFE1',
  //   height: 10717140,
  //   time: '2024-05-29T07:24:11.074548+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '41D8456BF42461728B0F1B4DEE791B9C181C5FBF4F855DF5617EC1FED7B29856',
  //   height: 688955,
  //   time: '2024-05-29T07:24:08.537919+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '30822FD11902BD8253FA9856F117F2A7EA23806BD4EC6D4BBFC52A4EA2A63523',
  //   height: 10717139,
  //   time: '2024-05-29T07:24:05.521632+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '704590B06C69981F3A6FDE054A01163BB155F9A2F8A7C40F0EB78313F9AAFAAB',
  //   height: 688954,
  //   time: '2024-05-29T07:24:02.425107+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'A93AFF08C8B0415BF5D7B964FEC16F05BCCA05DCE038DF8E8A8A9B87F4D3113E',
  //   height: 10717138,
  //   time: '2024-05-29T07:23:59.893072+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '55B980D7D6FB62CCD4C4D77ABADA4A97AC9081AC9C4516BC1FC74E61A4570508',
  //   height: 688953,
  //   time: '2024-05-29T07:23:56.139106+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '1D07BCA71E5E4BF39C187B41BD206783F1920377DCBD36E6D680056D6F229A50',
  //   height: 10717137,
  //   time: '2024-05-29T07:23:54.364397+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '6F8626C053DA691A8232E4539E8F9DD8CBCCB188261AF66FC20C16C35ABE54C5',
  //   height: 688952,
  //   time: '2024-05-29T07:23:49.975982+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '2E5DAEE534091B6750C948556C21B645E7EF5919ED4AAE90232B541DE7BE0640',
  //   height: 10717136,
  //   time: '2024-05-29T07:23:48.913398+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'D16FB638A900258845C706526C290810015DA2E6EE49FD62FBBACA27A435E58B',
  //   height: 688951,
  //   time: '2024-05-29T07:23:43.876727+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '6E0A4D0E67F31800BE4D21AA55F85CB3F14678A381BBB9033940966B91ED64DE',
  //   height: 10717135,
  //   time: '2024-05-29T07:23:43.331581+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'D55537E21E7A4F6BD6FF7B1D7763F21E9C40AA816E59F4F711828A79F833C7D3',
  //   height: 10717134,
  //   time: '2024-05-29T07:23:37.774509+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '1A61A84F77B4D85EA454BA99F2D33F915A47AA8BDF054E0BB6300F499B9384E4',
  //   height: 688950,
  //   time: '2024-05-29T07:23:37.704832+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'E00E4C11FFCB400BE7D9DDC9BEF15216E335FAEAF2056E7E6C37C95F93EA8554',
  //   height: 10717133,
  //   time: '2024-05-29T07:23:32.219321+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '5094C04DB0889F85739400B6042AC46F01EF4EEBE7B6F15B8025951167597AFF',
  //   height: 688949,
  //   time: '2024-05-29T07:23:31.558619+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '69212FEFED6EB6C06E4DCF4C32E97CF89BE5A576F921BAEE5FDFC183C06ECC0C',
  //   height: 10717132,
  //   time: '2024-05-29T07:23:26.698734+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '1DAC1F42EBAEFDAC4A1B2F67E28CBCED4F30E001B018FB866343C755EEEAC5A5',
  //   height: 688948,
  //   time: '2024-05-29T07:23:25.476715+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'AFB20E7DF25985245F642CCAE4DF02C0316738347DF84AD75C0D509F73B72276',
  //   height: 10717131,
  //   time: '2024-05-29T07:23:21.28838+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'DB1C8EFE57185C3A59BE527EFB405E3F629A3D57D9A4237B615050F0A05F2D8A',
  //   height: 688947,
  //   time: '2024-05-29T07:23:19.290846+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '1609E7D431990017CCB58DBD000D0B01AE0E7788C5DFD328DFB7EA6A73C19BCA',
  //   height: 10717130,
  //   time: '2024-05-29T07:23:15.732822+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '8A2EE8C1F180EDC632815E42A71A24AF91187D7FC1738CB353C8AD62A857A370',
  //   height: 688946,
  //   time: '2024-05-29T07:23:13.12563+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'BAEDFC8611BAC36FE1421AA58144E635BC0FEC80C16DCD7897DC753B378127EB',
  //   height: 10717129,
  //   time: '2024-05-29T07:23:10.126429+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '020CD4CD31EFE65C04FA40C432E250B06C13155EE91CE6C39BAA6795CA338426',
  //   height: 688945,
  //   time: '2024-05-29T07:23:06.949253+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '4889C5FD3CC117EB5C58270328A2B73E52DD90B6A5331416A5BF533442DC0A87',
  //   height: 10717128,
  //   time: '2024-05-29T07:23:04.548114+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'C0FF85115BA94615007F0734F1147464CA7E5D8D28317137D2AE7B06ABA47607',
  //   height: 688944,
  //   time: '2024-05-29T07:23:00.74106+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '7D248DF36D15863A2619D3706EE516E498A81F810697502753295028A867280C',
  //   height: 10717127,
  //   time: '2024-05-29T07:22:59.03895+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '5C83E9F741F74E7B49F24CF2ECEDC002F64D197F8DB40229E8AD4C528E851CFC',
  //   height: 688943,
  //   time: '2024-05-29T07:22:54.586854+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'F4D412AE179AE119F160FD8FAA39106F481F31B837F9BF21874FA84C342D10D6',
  //   height: 10717126,
  //   time: '2024-05-29T07:22:53.539326+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '268F73246B64E5E910FD1D64D9DED377650540B17D50A32F967CCC1AAEC8B19F',
  //   height: 688942,
  //   time: '2024-05-29T07:22:48.522815+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '251AE29C7C2ECE1F6DF1ADEBC519ACE87E5484543AC1F65DC6F8C92C538E41C3',
  //   height: 10717125,
  //   time: '2024-05-29T07:22:47.986802+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: 'E1EA7A974A30BA0D9E40DECE5B8D6FEA7B4B7E492FE8CDFBFD55784DE9224117',
  //   height: 688941,
  //   time: '2024-05-29T07:22:42.391299+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'ABFB8F196E133E072970AF2F574F3B0EEE3C5527554A2D35101B86096506B7DC',
  //   height: 10717124,
  //   time: '2024-05-29T07:22:42.378512+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'DCE8E0576D2A7DA72895ADFA4CD98C0C6006B8912CB1D2C450DBE70C4E184DBD',
  //   height: 10717123,
  //   time: '2024-05-29T07:22:36.852746+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '52ADFBD19EF422F8ADDA70E02D4D914DFA0421D958CB4D062FB807F881AC8273',
  //   height: 688940,
  //   time: '2024-05-29T07:22:36.166758+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: 'D36364107CFB709809827155BA5C229C06840FD72EA419C9B3CAA8A8D583199F',
  //   height: 10717122,
  //   time: '2024-05-29T07:22:31.315736+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '9242ED938334D6DE5CC48858BFA7606D7F50B27519FB30551CA24485C68CE2DF',
  //   height: 688939,
  //   time: '2024-05-29T07:22:30.184551+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '312F5C4FE0216A4D635F3D5C9E57980DE33909CF98673E402831F59ABBA98DFD',
  //   height: 10717121,
  //   time: '2024-05-29T07:22:25.755486+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '086F81799295B7DC27BD33C64DB17A7AB61273C93B457EF6415BF002351E9DFA',
  //   height: 688938,
  //   time: '2024-05-29T07:22:24.106443+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '4444EC18561923B1331506444DD8A7936A2BDAC1DD09784CD0D1ACFC4BE13395',
  //   height: 10717120,
  //   time: '2024-05-29T07:22:20.310288+00:00',
  // },
  // {
  //   chain_id: 4,
  //   hash: '92354578AA3070F4501D26E2C6562B525179C222650265A32B0F27B08CC24D1D',
  //   height: 688937,
  //   time: '2024-05-29T07:22:18.05944+00:00',
  // },
  // {
  //   chain_id: 5,
  //   hash: '634C834B8217798CE07F4E9AF22DF6793007B4233D928FD249D98595FCFD30FE',
  //   height: 10717119,
  //   time: '2024-05-29T07:22:14.811967+00:00',
  // },
  {
    chain_id: 4,
    hash: "B84DC1797A0FB3689A6A8B6834459309F6A16D50ECF79DBF60DA9D97F38F2959",
    height: 688936,
    time: "2024-05-29T07:22:12.032944+00:00"
  },
  {
    chain_id: 5,
    hash: "30229EE5609D7CD2F15ADE110765D7B11AF6E4309E3F122C76CED1B88AEFAAB7",
    height: 10717118,
    time: "2024-05-29T07:22:09.218245+00:00"
  },
  {
    chain_id: 4,
    hash: "0B0D3AE50C2728CA24270759046EFEE1A13ED0660F2065803571E6986C09B494",
    height: 688935,
    time: "2024-05-29T07:22:05.918295+00:00"
  },
  {
    chain_id: 5,
    hash: "8FE4BA6338881FB5442462912CB0F589428B098B9E250F30AF4014678D8EC960",
    height: 10717117,
    time: "2024-05-29T07:22:03.714725+00:00"
  },
  {
    chain_id: 4,
    hash: "362A07ACAD51E6DD48FD5C84536BEA5272D7479B17FAE90E778AD2B61E7C3BB8",
    height: 688934,
    time: "2024-05-29T07:21:59.783725+00:00"
  },
  {
    chain_id: 5,
    hash: "B75F14DCE958790C66DDEF38FA1821D06FD5AF6A098FE88FD526BCB5250D38B3",
    height: 10717116,
    time: "2024-05-29T07:21:58.204257+00:00"
  },
  {
    chain_id: 4,
    hash: "DC857F309F8FCC9A1C28F7AEF2A1E447B71E27BD7CFB3BF27678606F984ED57A",
    height: 688933,
    time: "2024-05-29T07:21:53.691279+00:00"
  },
  {
    chain_id: 5,
    hash: "03C40C2D8990A0666D912AD72785B6564743095B7FCA79E2076CEA233426A302",
    height: 10717115,
    time: "2024-05-29T07:21:52.683856+00:00"
  },
  {
    chain_id: 5,
    hash: "16A04CBBFD7A21E63767D171E5AAF932F4C447DFDCCCD6FFAD185DC2D70FCF02",
    height: 10717114,
    time: "2024-05-29T07:21:47.26671+00:00"
  },
  {
    chain_id: 4,
    hash: "C55B89C4632FE8BC9D969166EC9DC18F651CA8CECB9FAC4C76175FF6DEA51D18",
    height: 688932,
    time: "2024-05-29T07:21:47.058629+00:00"
  }
]

/**
 * we use this constructed type because importing the generated graphql types is too slow given the file size
 */
type CosmosBlock = Override<(typeof blockData)[0], { time: string }>

$: blocksStore = writable<Array<CosmosBlock>>(blockData as Array<CosmosBlock>)
$: if (blockData) {
  blocksStore.update(currentBlocks =>
    removeArrayDuplicates([...(blockData as Array<CosmosBlock>), ...currentBlocks], "height")
  )
}

const defaultColumns: Array<ColumnDef<CosmosBlock>> = [
  {
    accessorKey: "time",
    header: info => "Timestamp",
    meta: {
      class: "sticky"
    },
    cell: info =>
      flexRender(TimeElapsed, {
        timestamp: new Date(info.getValue() as string)
      })
  },
  {
    accessorKey: "height",
    header: info => "Height",
    accessorFn: row => row.height,
    cell: info =>
      flexRender(Button, {
        variant: "link",
        target: "_blank",
        value: info.getValue(),
        rel: "noopener noreferrer",
        class: "hover:cursor-pointer tabular-nums lining-nums",
        href: `https://api.testnet.bonlulu.uno/cosmos/base/tendermint/v1beta1/blocks/${info.getValue()}`
      })
  },
  {
    size: 1,
    accessorKey: "chain_id",
    header: info => "Chain ID",
    cell: info => CHAIN_MAP[info.getValue() as unknown as number].chainId
  },
  {
    accessorKey: "hash",
    header: info => "hash",
    cell: info =>
      flexRender(Button, {
        class: "p-0",
        variant: "link",
        target: "_blank",
        value: info.getValue(),
        rel: "noopener noreferrer",
        href: `https://rpc.testnet.bonlulu.uno/block_by_hash?hash=${info.getValue()}`
      })
  }
]

const options = writable<TableOptions<CosmosBlock>>({
  data: $blocksStore,
  enableHiding: true,
  enableFilters: true,
  columns: defaultColumns,
  autoResetPageIndex: true, // Automatically update pagination when data or page size changes
  enableColumnFilters: true,
  enableColumnResizing: true,
  enableMultiRowSelection: true,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getPaginationRowModel: getPaginationRowModel()
})

let virtualListElement: HTMLDivElement

const rerender = () =>
  options.update(options => ({ ...options, data: $blocksStore as unknown as Array<CosmosBlock> }))

const table = createSvelteTable(options)

$: blocksStore.subscribe(() => {
  if (!$blocksStore) return
  $table.setPageSize($blocksStore.length)
  rerender()
})

$: rows = $table.getRowModel().rows

$: virtualizer = createVirtualizer<HTMLDivElement, HTMLTableRowElement>({
  overscan: 20,
  count: rows.length,
  estimateSize: () => 34,
  getScrollElement: () => virtualListElement
})
</script>

<main class="mb-12 mt-10 flex size-full min-size-full flex-col items-center justify-center">
  <div class="rounded-md border-2 space-y-2 h-min max-h-[600px] overflow-auto w-6xl bg-card">
    <div
      bind:this={virtualListElement}
      class={cn('rounded-md border border-secondary border-solid')}
    >
      <Table.Root class={cn('size-full mx-auto rounded-md max-w-[1000px] overflow-auto')}>
        <Table.Header
          class={cn('outline outline-1 outline-secondary sticky top-0 left-0 bottom-0 z-50')}
        >
          {#each $table.getHeaderGroups() as headerGroup (headerGroup.id)}
            <Table.Row class="font-bold text-md sticky">
              {#each headerGroup.headers as header (header.id)}
                <Table.Head
                  colspan={header.colSpan}
                  class={cn(
                    //
                    'text-left px-2 sticky top-0',
                    `w-[${header.getSize()}px]`,
                  )}
                >
                  {#if !header.isPlaceholder}
                    <Button
                      variant="ghost"
                      disabled={!header.column.getCanSort()}
                      on:click={header.column.getToggleSortingHandler()}
                      class={cn('cursor-pointer select-none capitalize')}
                    >
                      <svelte:component
                        this={flexRender(header.column.columnDef.header, header.getContext())}
                      />
                    </Button>
                  {/if}
                </Table.Head>
              {/each}
            </Table.Row>
          {/each}
        </Table.Header>
        <Table.Body class={cn('relative', `h-[${$virtualizer.getTotalSize()}px] w-6xl`)}>
          {#each $virtualizer.getVirtualItems() as row, index (row.index)}
            <Table.Row
              class={cn(
                'h-5 text-left overflow-auto',
                'border-b-[1px] border-solid border-secondary',
                // index % 2 === 0 ? 'bg-background' : 'border-gray-950',
              )}
            >
              <!-- {index} -->
              {#each rows[row.index].getVisibleCells() as cell, index (cell.id)}
                <Table.Cell
                  class={cn(
                    //
                    'px-2 py-0 text-left',
                    // `w-[${cell.column.columnDef.size}]`,
                  )}
                >
                  <svelte:component
                    this={flexRender(cell.column.columnDef.cell, cell.getContext())}
                  />
                </Table.Cell>
              {/each}
            </Table.Row>
          {/each}
        </Table.Body>
      </Table.Root>
    </div>
  </div>
  <!-- <div class="w-full max-w-[925px] flex items-center justify-between p-2">
    <div class="flex-1 text-sm text-muted-foreground">300 rows</div>
    <div class="flex items-center space-x-6 lg:space-x-8">
      <div class="flex items-center space-x-2">
        <p class="text-sm font-medium">Rows per page</p>
        <Select.Root
          onSelectedChange={selected => $table.setPageSize(Number(selected?.value))}
          selected={{ value: 10, label: '10' }}
        >
          <Select.Trigger class="h-8 w-[70px]">
            <Select.Value placeholder="Select page size" />
          </Select.Trigger>
          <Select.Content
            sideOffset={8}
            transition={flyAndScale}
            class="outline outline-[1px] outline-accent"
          >
            <Select.Item value="10">10</Select.Item>
            <Select.Item value="20">20</Select.Item>
            <Select.Item value="30">30</Select.Item>
            <Select.Item value="40">40</Select.Item>
            <Select.Item value="50">50</Select.Item>
          </Select.Content>
        </Select.Root>
      </div>
      <div class="flex w-[75px] items-center justify-center text-sm font-medium">1 of 3</div>
      <div class="flex items-center space-x-2">
        <Button variant="outline" class="hidden h-8 w-8 p-0 lg:flex" on:click={() => {}}>
          <span class="sr-only">Go to first page</span>
          <DoubleArrowLeft class="size-7" />
        </Button>
        <Button variant="outline" class="h-8 w-8 p-0">
          <span class="sr-only">Go to previous page</span>
          <ChevronLeft class="size-7" />
        </Button>
        <Button variant="outline" class="h-8 w-8 p-0">
          <span class="sr-only">Go to next page</span>
          <ChevronRight class="size-7" />
        </Button>
        <Button variant="outline" class="hidden h-8 w-8 p-0 lg:flex">
          <span class="sr-only">Go to last page</span>
          <DoubleArrowRight class="size-7" />
        </Button>
      </div>
    </div>
  </div>

  <div class="w-full max-w-[925px] px-2 flex justify-between">
    <p class="text-white/70 text-sm">{blockData.length} total rows</p>
    <div class="flex">
      <p class="w-max">Rows per page</p>
      <Select.Root>
        <Select.Trigger class="px-3">
          <Select.Value placeholder="Theme" class="pr-2" />
        </Select.Trigger>
        <Select.Content class="outline-union-accent-500/50">
          <Select.Item value="light">Light</Select.Item>
          <Select.Item value="dark">Dark</Select.Item>
          <Select.Item value="system">System</Select.Item>
        </Select.Content>
      </Select.Root>
    </div> 
  </div>-->
</main>
