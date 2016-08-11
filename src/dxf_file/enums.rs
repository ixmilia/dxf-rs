// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate num;

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Dxf3DDwfPrecision {
    Deviation_1 = 1,
    Deviation_0_5 = 2,
    Deviation_0_2 = 3,
    Deviation_0_1 = 4,
    Deviation_0_01 = 5,
    Deviation_0_001 = 6,
}
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum DxfAcadVersion {
    Version_1_0,
    Version_1_2,
    Version_1_40,
    Version_2_05,
    Version_2_10,
    Version_2_21,
    Version_2_22,
    Version_2_5,
    Version_2_6,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R2000,
    R2004,
    R2007,
    R2010,
    R2013,
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfAngleDirection {
    CounterClockwise = 0,
    Clockwise = 1,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfAngleFormat {
    DecimalDegrees = 0,
    DegreesMinutesSeconds = 1,
    Gradians = 2,
    Radians = 3,
    SurveyorsUnits = 4,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfAttachmentPoint {
    TopLeft = 1,
    TopCenter = 2,
    TopRight = 3,
    MiddleLeft = 4,
    MiddleCenter = 5,
    MiddleRight = 6,
    BottomLeft = 7,
    BottomCenter = 8,
    BottomRight = 9,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfAttributeVisibility {
    None = 0,
    Normal = 1,
    All = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfBackgroundFillSetting {
    Off = 0,
    UseBackgroundFillColor = 1,
    UseDrawingWindowColor = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfBottomTextAttachmentDirection {
    Center = 9,
    UnderlineAndCenter = 10,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfCoordinateDisplay {
    Static = 0,
    ContinuousUpdate = 1,
    DistanceAngleFormat = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfDimensionArcSymbolDisplayMode {
    SymbolBeforeText = 0,
    SymbolAboveText = 1,
    Suppress = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfDimensionAssociativity {
    NoAssociationExploded = 0,
    NonAssociativeObjects = 1,
    AssociativeObjects = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfDimensionFit {
    TextAndArrowsOutsideLines = 0,
    MoveArrowsFirst = 1,
    MoveTextFirst = 2,
    MoveEitherForBestFit = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfDimensionFractionFormat {
    HorizontalStacking = 0,
    DiagonalStacking = 1,
    NotStacked = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfDimensionTextBackgroundColorMode {
    None = 0,
    UseDrawingBackground = 1,
    Custom = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfDimensionTextJustification {
    AboveLineCenter = 0,
    AboveLineNextToFirstExtension = 1,
    AboveLineNextToSecondExtension = 2,
    AboveLineCenteredOnFirstExtension = 3,
    AboveLineCenteredOnSecondExtension = 4,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfDimensionTextMovementRule {
    MoveLineWithText = 0,
    AddLeaderWhenTextIsMoved = 1,
    MoveTextFreely = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfDimensionType {
    RotatedHorizontalOrVertical = 0,
    Aligned = 1,
    Angular = 2,
    Diameter = 3,
    Radius = 4,
    AngularThreePoint = 5,
    Ordinate = 6,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfDragMode {
    Off = 0,
    On = 1,
    Auto = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfDrawingDirection {
    LeftToRight = 1,
    TopToBottom = 3,
    ByStyle = 5,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfDrawingUnits {
    English = 0,
    Metric = 1,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfEndCapSetting {
    None = 0,
    Round = 1,
    Angle = 2,
    Square = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfFontType {
    TTF = 0,
    SHX = 1,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfHelixConstraint {
    ConstrainTurnHeight = 0,
    ConstrainTurns = 1,
    ConstrainHeight = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfHorizontalTextJustification {
    Left = 0,
    Center = 1,
    Right = 2,
    Aligned = 3,
    Middle = 4,
    Fit = 5,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfImageClippingBoundaryType {
    Rectangular = 1,
    Polygonal = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfJoinStyle {
    None = 0,
    Round = 1,
    Angle = 2,
    Flat = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfJustification {
    Top = 0,
    Middle = 1,
    Bottom = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfLayerAndSpatialIndexSaveMode {
    None = 0,
    LayerIndex = 1,
    SpatialIndex = 2,
    LayerAndSpatialIndex = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfLeaderCreationAnnotationType {
    WithTextAnnotation = 0,
    WithToleranceAnnotation = 1,
    WithBlockReferenceAnnotation = 2,
    NoAnnotation = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfLeaderHooklineDirection {
    OppositeFromHorizontalVector = 0,
    SameAsHorizontalVector = 1,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfLeaderPathType {
    StraightLineSegments = 0,
    Spline = 1,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfLightAttenuationType {
    None = 0,
    InverseLinear = 1,
    InverseSquare = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfLightType {
    Distant = 1,
    Point = 2,
    Spot = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfLinetypeStyle {
    Off = 0,
    Solid = 1,
    Dashed = 2,
    Dotted = 3,
    ShortDash = 4,
    MediumDash = 5,
    LongDash = 6,
    DoubleShortDash = 7,
    DoubleMediumDash = 8,
    DoubleLongDash = 9,
    MediumLongDash = 10,
    SparseDot = 11,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfLoftedObjectNormalMode {
    Ruled = 0,
    SmoothFit = 1,
    StartCrossSection = 2,
    EndCrossSection = 3,
    StartAndEndCrossSections = 4,
    AllCrossSections = 5,
    UseDraftAngleAndMagnitude = 6,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfMTextFlag {
    MultilineAttribute = 2,
    ConstantMultilineAttributeDefinition = 4,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfMTextLineSpacingStyle {
    AtLeast = 1,
    Exact = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfNonAngularUnits {
    Scientific = 1,
    Decimal = 2,
    Engineering = 3,
    Architectural = 4,
    Fractional = 5,
    WindowsDesktop = 6,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfOleObjectType {
    Link = 1,
    Embedded = 2,
    Static = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfOrthographicViewType {
    None = 0,
    Top = 1,
    Bottom = 2,
    Front = 3,
    Back = 4,
    Left = 5,
    Right = 6,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfPickStyle {
    None = 0,
    Group = 1,
    AssociativeHatch = 2,
    GroupAndAssociativeHatch = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfPlotStyle {
    ByLayer = 0,
    ByBlock = 1,
    ByDictionaryDefault = 2,
    ByObjectId = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfPolylineCurvedAndSmoothSurfaceType {
    None = 0,
    QuadraticBSpline = 5,
    CubicBSpline = 6,
    Bezier = 8,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfPolySketchMode {
    SketchLines = 0,
    SketchPolylines = 1,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfShadeEdgeMode {
    FacesShadedEdgeNotHighlighted = 0,
    FacesShadedEdgesHighlightedInBlack = 1,
    FacesNotFilledEdgesInEntityColor = 2,
    FacesInEntityColorEdgesInBlack = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfShadowMode {
    CastsAndReceivesShadows = 0,
    CastsShadows = 1,
    ReceivesShadows = 2,
    IgnoresShadows = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfSnapIsometricPlane {
    Left = 0,
    Top = 1,
    Right = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfSnapStyle {
    Standard = 0,
    Isometric = 1,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfSolidHistoryMode {
    None = 0,
    DoesNotOverride = 1,
    Override = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfTextAttachmentDirection {
    Horizontal = 0,
    Vertical = 1,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfTextDirection {
    LeftToRight = 0,
    RightToLeft = 1,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfTextLineSpacingStyle {
    AtLeast = 1,
    Exact = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfTileModeDescriptor {
    InTiledViewport = 0,
    InNonTiledViewport = 1,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum DxfTimeZone {
    InternationalDateLineWest = -12000,
    MidwayIsland_Samoa = -11000,
    Hawaii = -10000,
    Alaska = -9000,
    PacificTime_US_Canada_SanFrancisco_Vancouver = -8000,
    Arizona = -7000,
    //Chihuahua_LaPaz_Mazatlan = -7000,
    //MountainTime_US_Canada = -7000,
    Mazatlan = -7002,
    CentralAmerica = -6000,
    CentralTime_US_Canada = -6001,
    Guadalajara_MexicoCity_Monterrey = -6002,
    Saskatchewan = -6003,
    EasternTime_US_Canada_ = -5000,
    Indiana_East_ = -5001,
    Bogota_Lima_Quito = -5002,
    AtlanticTime_Canada_ = -4000,
    Caracas_LaPaz = -4001,
    Santiago = -4002,
    Newfoundland = -3300,
    Brasilia = -3000,
    BuenosAires_Georgetown = -3001,
    Greenland = -3002,
    MidAtlantic = -2000,
    Azores = -1000,
    CapeVerdeIs = -1001,
    UniversalCoordinatedTime = 0,
    GreenwichMeanTime = 1,
    Casablanca_Monrovia = 2,
    Amsterdam_Berlin_Bern_Rome_Stockholm = 1000,
    Brussels_Madrid_Copenhagen_Paris = 1001,
    Belgrade_Bratislava_Budapest_Ljubljana_Prague = 1002,
    Sarajevo_Skopje_Warsaw_Zagreb = 1003,
    WestCentralAfrica = 1004,
    Athens_Beirut_Istanbul_Minsk = 2000,
    Bucharest = 2001,
    Cairo = 2002,
    Harare_Pretoria = 2003,
    Helsinki_Kyiv_Sofia_Talinn_Vilnius = 2004,
    Jerusalem = 2005,
    Moscow_StPetersburg_Volograd = 3000,
    Kuwait_Riyadh = 3001,
    Baghdad = 3002,
    Nairobi = 3003,
    Tehran = 3300,
    AbuDhabi_Muscat = 4000,
    Baku_Tbilisi_Yerevan = 4001,
    Kabul = 4300,
    Ekaterinburg = 5000,
    Islamabad_Karachi_Tashkent = 5001,
    Chennai_Kolkata_Mumbai_NewDelhi = 5300,
    Kathmandu = 5450,
    Almaty_Novosibirsk = 6000,
    Astana_Dhaka = 6001,
    SriJayawardenepura = 6002,
    Rangoon = 6300,
    Bangkok_Hanoi_Jakarta = 7000,
    Krasnoyarsk = 7001,
    Beijing_Chongqing_HongKong_Urumqi = 8000,
    KualaLumpur_Singapore = 8001,
    Taipei = 8002,
    Irkutsk_UlaanBataar = 8003,
    Perth = 8004,
    Osaka_Sapporo_Tokyo = 9000,
    Seoul = 9001,
    Yakutsk = 9002,
    Adelaide = 9300,
    Darwin = 9301,
    Canberra_Melbourne_Sydney = 10000,
    Guam_PortMoresby = 10001,
    Brisbane = 10002,
    Hobart = 10003,
    Vladivostok = 10004,
    Magadan_SolomonIs_NewCaledonia = 11000,
    Auckland_Wellington = 12000,
    Fiji_Kamchatka_MarshallIs = 12001,
    Nukualofa_Tonga = 13000,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfTopTextAttachmentDirection {
    Center = 9,
    OverlineAndCenter = 10,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfUnderlayFrameMode {
    None = 0,
    DisplayAndPlot = 1,
    DisplayNoPlot = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfUnitFormat {
    Scientific = 1,
    Decimal = 2,
    Engineering = 3,
    ArchitecturalStacked = 4,
    FractionalStacked = 5,
    Architectural = 6,
    Fractional = 7,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfUnits {
    Unitless = 0,
    Inches = 1,
    Feet = 2,
    Miles = 3,
    Millimeters = 4,
    Centimeters = 5,
    Meters = 6,
    Kilometers = 7,
    Microinches = 8,
    Mils = 9,
    Yards = 10,
    Angstroms = 11,
    Nanometers = 12,
    Microns = 13,
    Decimeters = 14,
    Decameters = 15,
    Hectometers = 16,
    Gigameters = 17,
    AstronomicalUnits = 18,
    LightYears = 19,
    Parsecs = 20,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfVerticalTextJustification {
    Baseline = 0,
    Bottom = 1,
    Middle = 2,
    Top = 3,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfXrefClippingBoundaryVisibility {
    NotDisplayedNotPlotted = 0,
    DisplayedAndPlotted = 1,
    DisplayedNotPlotted = 2,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DxfUnitZeroSuppression {
    SuppressZeroFeetAndZeroInches = 0,
    IncludeZeroFeetAndZeroInches = 1,
    IncludeZeroFeetAndSuppressZeroInches = 2,
    IncludeZeroInchesAndSuppressZeroFeet = 3,
}
}
