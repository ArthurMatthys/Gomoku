extern crate rand;

use super::super::render::board;

const RDM_NUMBERS: [u64; 722] = [13785793976861693916, 17289606658829237898, 14410926314751103959, 15355535775449527895, 5957302548081304230, 4107162304072211205, 17777452486032802245, 18015724369326906265, 7343802154322436360, 1944492544674060650, 14432591684003608359, 13607738295376943590, 13877628732604689765, 16105693781099222965, 5401995763311616360, 11904952687354276773, 6219051558070779065, 14562256297059813243, 9690525685916098100, 16279272984172293445, 7324423297393430080, 10476361460690642603, 3078911696604190901, 4861524869273472207, 17416640613493496924, 10118722947613843451, 934491545998234204, 14619945177794263398, 1453659670832861128, 17287170172360551339, 18417591713518866117, 4363611956955451035, 16405791010142533109, 18129990129380767154, 8871519261777434331, 17690612556846470001, 4300464544869525539, 12999356155615120544, 17928558038666258189, 6504330214708993971, 15901668224435444329, 10319438467594742623, 7023194256957247860, 4750348519688685318, 8541472734884113262, 57593666915972565, 4825031464807279948, 7924243278525733382, 901405703763796340, 3231075656263062203, 7845882154880206722, 2693770129818451237, 8229908229438528862, 17775111331302515625, 475577464172804710, 9042588221174076945, 10712853085549315715, 14480108583123766812, 13133042685183835031, 8079899426360799020, 1956225082854658401, 5108337880378882898, 10103311700730388337, 54965166108586151, 10269785652853797563, 9628594617964115699, 13547739171803841588, 13898517346655228317, 723570026988184231, 1879684883656716692, 13047380345361538759, 2405619113871669581, 13939066426829290153, 929400172435917808, 5866276119434118356, 17576129166550943933, 5184802156752266258, 14159600787628182112, 12179764664633508016, 15890084289449631788, 14004034911239305539, 11354497828376188342, 6892642433385127411, 2635794374526595438, 2218330301020282168, 12603711980911508082, 16645544775805402038, 13753239705037175769, 10327447098334685021, 15091557963393020982, 11611691865727195019, 717696739621931136, 15841558720610214824, 4013511726090053548, 13797251013274600903, 10515133924310237720, 1339608019333210055, 2462160611257335060, 7523658374846238348, 9263995207215113395, 7489297064767514976, 5333919196908965807, 9244317551945091573, 15056820889877841055, 6663373856882341911, 827659313303893660, 15101128473279209805, 4475168953871048207, 4994126799031459657, 8871504426341970024, 15267814453387887528, 8790143930306127691, 15150022396880867979, 4769538981607913455, 11669690301597364994, 11619675126428612421, 6144746400412295676, 3007384272458024041, 16817829595353642798, 17631871594711688648, 9347825899466842126, 665492308447723732, 5537308973127298401, 13389802661140215022, 2826378902186880264, 2176000900353958676, 15392645848763851832, 16026208457443080962, 14283850862470900757, 2679880596278380370, 455752580309504143, 1753674249358960327, 14543423284815744741, 11781739247288905860, 1703323658917547500, 446043161954422437, 15701818990607530617, 553959628236539229, 1307808848459041697, 5286325274027421802, 10497717907304241962, 2366924268532812115, 5704834858901623863, 10004537920980941272, 12984255551940624861, 14834412637693767598, 12162450366694198501, 16029961070906747344, 6336601085928532932, 12529917215063786237, 7812019637695889006, 16592018679248978751, 8863178634160343840, 14151013259878491139, 5273020876090527023, 9496605946822022652, 17560934459948116342, 6672351917365719209, 6474379777593143084, 4380489106514280387, 11055174492041492552, 15339595561596703315, 8849072965928578872, 6081036822863674912, 6508638860094157110, 14964460337021138649, 8482595074191961942, 3646310968084542717, 18176110048513183656, 18274461825854272250, 10789701911195642251, 1554008654368694012, 349451829199978244, 13134360260871945112, 2795854337585541249, 17650311954241272468, 15089877305495682252, 463919671692128257, 8512371665989049344, 18265294441157544992, 1425571233992797110, 11371569599690290477, 930589698350934764, 7818061699167092343, 6906250623714739496, 13412275934070701729, 15718066825040109089, 1286253053946782930, 13337563813550175565, 5966041764349596664, 12542501722356600586, 5162165153089553259, 10703754454515898433, 5682873237637624271, 5804407008527558973, 2407852861272675689, 217455586623594200, 5779968560418914504, 11504426860384235631, 5333782580275905019, 9994947605220246928, 1300925643015891448, 705693134426098680, 4966692782828749773, 14715264561332379555, 5792726490157542668, 2317114706384554703, 7793572734371795610, 16301416992332627222, 17795149463188841318, 13243697532787180600, 338631634135204489, 1666206868376298068, 1026099139953347480, 14406262705949678660, 7400881456597754652, 10840291826879022278, 18030236225608962690, 367676295356146147, 18386779581730810231, 15504911825025160429, 165462324849282387, 14647582625572093070, 7095869558051475586, 4845951510204651147, 5404193527764504023, 16111689937309896469, 3086746931593199294, 18338184706597669164, 7750836641792659299, 13652955505230263126, 12490870247680961809, 5574276327902237332, 4609694744526773592, 1466553183062614811, 6975015749617134140, 10489151025827981855, 6852646609852434311, 11045149910716188290, 11590368111569122609, 18215705055301247223, 15986629031585563062, 16386442107702341647, 5232378323931264908, 13294450611038149360, 5965070707302576329, 5226759814993229962, 18270220526221660241, 11298688848198590564, 9166590766672414075, 9548719374253091855, 11604164685397343640, 12955659003087126910, 11229911135576845706, 10509115133685773585, 6435460164704325027, 140351811870339480, 5496356496307829298, 7680083228947461146, 6976351814637547645, 7974503797933607545, 17111399485409199237, 6243895743968774927, 13008705908779243362, 9657392279657046517, 556992304383472768, 13784768242222599299, 12365897756585427543, 2644650509090256430, 9354460624098114261, 15146288471651070010, 13682624407621231089, 3195149941753292732, 8818949088279718895, 14636864882773853795, 1071666515444445706, 15830459349636019799, 8349223626209518600, 11166899875390376567, 545583356197031338, 10211432650399788103, 11382809458857533055, 13008096167121609764, 8387220580232395855, 9430717615169634239, 16951205839734323912, 18442578514152366545, 15664144733008058449, 7630590819929455431, 292000943492464471, 11113768899167602850, 4175092301799025633, 7078314506182586150, 10658440741688759960, 2853524970912910565, 612161905925007627, 7039702501340411961, 1114555537305256100, 849850089393490268, 531346098468254564, 13704489226099210618, 10650893120351928853, 8923530493407941904, 2969074690520272811, 16894117222239531075, 1439177520892561877, 945343396426082509, 4761040180891599418, 12564930285129143608, 13593341978639336023, 7307515330742310024, 1564973158724907030, 7493607721896946831, 9340028512879527236, 16493235840111106100, 2019591519635087573, 2503438755281059428, 18096372600394896255, 12424289230089708215, 2372448623130217823, 11730587782560559427, 16225064218590620424, 3869029600057068381, 3920865541581102980, 13849463974374842252, 8811415972114865454, 9991082938076956653, 5730238890733307249, 4782461071078227036, 17645661812395022387, 17790772947819598164, 6347462675147966881, 10460415809118703482, 6217473995339621751, 10183932917197247748, 8856495409353018977, 11896328606250960300, 15827001490394319196, 6993328372792565720, 15732337971847315497, 331677423522971956, 15524810693911400005, 438711489641786331, 17718288030060135775, 9621963822483422212, 6878557120388656594, 8556864975277142721, 5712876925718819068, 2519910895102303031, 16846886592709083084, 15054692012583270667, 7351117248904602870, 17980647706402296194, 16429872514297046720, 10031848086733335328, 15946501181578583951, 11838444472783774260, 9986371761320621433, 7679241448555029746, 10054841789442653492, 7454965419140863853, 15779949452469549810, 9223228552762749869, 4749486658661050703, 18002317691391468126, 3094921502189978299, 1009785247039519678, 9101790106998107427, 10336437505413585424, 16483820611994999233, 13269615819415838632, 7495134657179033964, 11765370048125686736, 168687281898695587, 3188090210020428208, 17885259754449426440, 9596148064661639128, 17319465520916330114, 13745201991851926776, 657235686364023818, 17204746593373676388, 3551109947797268321, 8314039856835396809, 12987049939828785852, 8022863018921506436, 8611438125572842233, 13915837265687956294, 7256183588138629775, 2741889302397801888, 13002954083939034239, 16094853889065470928, 4943014699947458014, 6722878158782162987, 17184577707238596508, 16709962129390413622, 6487120450311018634, 11659024750307676174, 7073936845337770361, 5845729503428285580, 2570856256866915302, 16408682798920915382, 11618698977405926939, 7255587122062534044, 15569572112995218377, 9384116878651033582, 10143536776948067816, 17521742609811961039, 5052903752316516193, 10012116908972091620, 7071599583775761678, 13402247261471388372, 5681911284010141005, 343174688877933005, 1361444221677225543, 16603434699623123669, 10831585067179007188, 7635399919148444005, 16698515159629847694, 5305019945190317558, 10061962680058744567, 14056828736344299980, 14005863590581528040, 10158087995858616530, 6991175826835534487, 13374153821126282623, 7644946155704554032, 8274087855791713028, 2851490095273719711, 15659522740145270517, 499940703688444901, 3820376144015491189, 2572357284071628406, 17836645262073999538, 7911707293295244715, 10556309110444526304, 13907049909701332807, 98711362832136131, 10928899443525190101, 17957761588257014439, 5750755920119054669, 16761891386166627212, 8363572693109857922, 12404987950616840894, 8699102832909542790, 18023087179991486080, 8489627095722217718, 12940066845509420910, 3133854581491122986, 844718255214332638, 11913649177748452037, 4489165805581923407, 17655700074294268358, 2775063135848091529, 10445263600948292802, 9363705182601056513, 6422959783621119084, 17323064805669900768, 16479226946651698309, 8175511142816338974, 8232088827919728148, 3154165347080777169, 14792537563216600852, 12695921804134420068, 3572142568237965552, 7027454220745399434, 17928554634551510035, 10704241799668118266, 18032160370555516824, 17961755037619935040, 15457577833782607062, 3378591332769321111, 3601492215144494461, 11140660254475806394, 17765436502499736377, 15388369773744927805, 13873330382185085083, 17574611563462663469, 7481336272479411795, 3977775026645227924, 8676556369762812733, 17356261680526674093, 6434533801922120464, 17638676028064920978, 14649155223225863851, 17454265632132229301, 5753827401383718732, 8312965014033259479, 11117863840304471291, 1966939531200667355, 10655578594101477849, 3417806386246610656, 5532612817671165422, 8228212151268264828, 14924833125893961785, 12517441018109864431, 11427525398136811389, 17429965210403221677, 11475677803830608788, 4349030574843908820, 529781839938130298, 3587603840977600351, 10622539142137915663, 1738270169734055164, 16160655213123642347, 12952743258769826990, 14297207349294079204, 16741899661718269021, 2064411356514514696, 8443365856431545043, 4084862669043912943, 16268108761631558061, 10171471374466999141, 7298073256692031211, 3690012829683259473, 17589675196799319254, 14378069968931675656, 5274880484665086833, 330601226360413013, 12144081925769729896, 992759634443139865, 4041908095504951985, 6692120947992123922, 9006608408219261692, 17370222400444538428, 13738998791702049036, 7344940151710634996, 3407562612871722839, 7509255844572971216, 15619686426540225498, 2297883952467862319, 5042700132115390179, 8643681327083546998, 16242976271293740658, 17457138755504927821, 8550703243416535258, 15238699653979918273, 18325710939715932797, 12318302370370938718, 8279732590963450516, 17269895255984281056, 7186655029185786411, 5001927791658420372, 4416532478687590672, 14750877762489306344, 7636480938686722860, 843174574102066071, 6661262057817995649, 371716935906193238, 12930312492917732453, 16696051417311379777, 3056823655564987982, 656080687611367595, 9394203906614465330, 5281331520207889847, 8585992621643829557, 3254711473540085578, 16307741824796232448, 12200656730443442423, 5342563469283523422, 16634686848475089855, 17137367321903154222, 5091477082519233310, 4921726206501172316, 16700933471101865697, 14422112124352546193, 12600598276676665273, 5716523405625016625, 11964821213099375850, 5512216387821855234, 11962403245047022398, 4655788090161495831, 508500663497741367, 11908721538339615931, 2071066303619306225, 5719168017945210041, 12526114470066792883, 5919352130797032558, 10676111411826330350, 6957257190737420992, 2419363712246104029, 5379435505615517784, 4459720890412228512, 6901039664323426257, 4532770438763486087, 14300373156313820063, 808182940487444369, 981024206930228413, 11946955241446234856, 17778532537494884639, 6990070984112236657, 11538920367992264312, 5546920707116500212, 11871948772270305808, 13449809862509714682, 666943623922531601, 13477981083366459961, 3288649510613261017, 4727471907003373513, 6466101701957343437, 13052779813090810883, 10755342044029236663, 6708546534165670130, 2813666140965363417, 12809459114039662207, 8784385406264378637, 15068882986756206558, 11432850784061500850, 5305707382825158301, 15507836579110220219, 12644727484528616096, 5174621455160851462, 12021264094287281848, 14633676495921386782, 10289278603603079278, 16322989746496061493, 2226690029676996438, 6591620929199057124, 9792186053709116333, 13483325743658317170, 16152305265607100840, 10167530646880662258, 7695099809869296230, 16629651079834705388, 4987143838726622466, 14101952137675379544, 10898877369728580525, 13556480557306226209, 4037018264081260731, 7790080178370606988, 5902371357532230374, 6210411563467442036, 897070039995238547, 10580163677678654687, 15341552630279213953, 11413053169252820001, 7435386430533270046, 16293532426710910553, 4534416715404412628, 8741219770179607222, 17456316870183070972, 11585136694374439470, 3380367982293624037, 5202489752864107237, 12347544671305960930, 15115622840016659583, 12247649296431164792, 16073397293046823045, 11053131196012374597, 14029826514380512909, 3183203556547921947, 9858895877235351181, 7438733365127053435, 3166732900285644466, 2325628897688406476, 5764320821681594878, 4181232008858832964, 8701535560391176110, 16680298924132389204, 7073429318251588369, 5163810707257982772, 7468956944170733212, 16461998605083947597, 792558868747337091, 10700715935592395744, 11919080489289246964, 11457775629591023263, 14114730525248638909, 5703521977640884344, 7386905271357019932, 2178318687584533709, 11846468972403015361, 14134577206007743459, 16670572443651221070, 12371713886408433679, 3102229216261144752, 842649279535486707, 13308999563668898589, 10606049129335090867, 4676807036495663294, 6339387467116012281, 7223082034157330995, 14157184160046481998, 2394352239598073780, 17075812771654825535, 15555000450885462369, 2408456590618678534, 9115371058659682594, 3290939264817946485, 8703755690437238518, 2580754034006070411, 12946002012645320672, 14899834143186316379, 16946346228304641832, 10162809453249563340, 17495218249654890985, 11867362031956239318, 13080900843654444439, 7771241358693808564, 15645792866521425431, 14992754914706933125, 10766452395056928291, 9028044894178659207, 12490915999344728232, 9746276464759107824, 6503829525155167774, 12782389639992404542, 3380926849202557861, 5929987935717355045, 8297164483886264282, 4129161079219639830, 17941586210789887555, 12996170639510458134, 7897055552702289547, 4706099345985571722, 831669234290077166, 4014155857945698073, 6647564319976345654, 18329492236893215403, 10145417482812860176, 2774279107047607750, 1889970738766135425, 15793339727605457095];

// Type of element in TT
#[derive(Copy, Clone, PartialEq)]
pub enum TypeOfEl {
    Lowerbound,
    Upperbound,
    Exact,
}

// Transposition table
#[derive(Clone, Copy)]
pub struct TT {
    pub is_valid: bool,
    // Zobrist key, to check for collision
    pub key: u64,
    // Values
    pub value: i64,
    pub r#type: TypeOfEl,
    pub depth: i8,
    pub r#move: Option<(usize, usize)>,
}

// Transposition table of at least 2^20 entries
// 2^22 here => 4 194 304 entries, from which we found
// the next prime : 4194319 (to avoid hash collision)
pub fn initialize_transposition_table() -> Vec<TT> {
    let initialized_struct = TT {
        key: 0,
        is_valid: false,
        value: 0,
        r#type: TypeOfEl::Exact,
        depth: 0,
        r#move: None,
    };
    vec![initialized_struct; 4194319]
}

pub fn retrieve_tt_from_hash(tt: &Vec<TT>, zhash: &u64) -> TT {
    tt[(*zhash % tt.len() as u64) as usize]
}

pub fn store_tt_entry(
    tt: &mut Vec<TT>,
    zhash: &mut u64,
    tte: TT
) -> () {
    let len = tt.len();
    tt[(*zhash % len as u64) as usize] = tte;
}

// Zobrist hash
pub const ZPIECES: [usize; 2] = [0, 1]; // 0 is black_pawn, 1 is white_pawn

// Initialize the first zobrist hash
// We initialize a 3D array of 19x19 containing for each cell
// An array of uniform random f64 number (2, one for each piece)
pub fn init_zboard() -> [[[u64; 2]; board::SIZE_BOARD]; board::SIZE_BOARD] {
    let mut table = [[[0_u64; 2]; board::SIZE_BOARD]; board::SIZE_BOARD];

    let mut index = 0;
    for line in 0..board::SIZE_BOARD {
        for col in 0..board::SIZE_BOARD {
            for i in 0..2 {
                // Fill it with a uniformly generated f64 to avoid collision
                table[line][col][i] = RDM_NUMBERS[index];
                index += 1;
            }
        }
    }
    table
}

// Function that initializes the zhash as a u64 accordingly to the current board's state
pub fn board_to_zhash(
    board: &[[Option<bool>; board::SIZE_BOARD]; board::SIZE_BOARD],
    ztable: &[[[u64; 2]; board::SIZE_BOARD]; board::SIZE_BOARD]
) -> u64 {
    let mut hash: u64 = 0;

    for line in 0..board::SIZE_BOARD {
        for col in 0..board::SIZE_BOARD {
            match board[line][col] {
                None => (),
                Some(true) => hash ^= ztable[line][col][ZPIECES[1]],
                Some(false) => hash ^= ztable[line][col][ZPIECES[0]],
            }
        }
    }
    hash
}
