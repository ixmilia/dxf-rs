use crate::{DxfError, DxfResult};
use std::fmt;

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum Dwf3DPrecision {
    Deviation_1 = 1,
    Deviation_0_5 = 2,
    Deviation_0_2 = 3,
    Deviation_0_1 = 4,
    Deviation_0_01 = 5,
    Deviation_0_001 = 6,
}
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum AcadVersion {
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
    R2018,
}

impl AcadVersion {
    pub fn from(val: String) -> DxfResult<AcadVersion> {
        match &*val {
            "MC0.0" => Ok(AcadVersion::Version_1_0),
            "AC1.2" => Ok(AcadVersion::Version_1_2),
            "AC1.40" => Ok(AcadVersion::Version_1_40),
            "AC1.50" => Ok(AcadVersion::Version_2_05),
            "AC2.10" => Ok(AcadVersion::Version_2_10),
            "AC2.21" => Ok(AcadVersion::Version_2_21),
            "AC2.22" => Ok(AcadVersion::Version_2_22),
            "AC1001" => Ok(AcadVersion::Version_2_22),
            "AC1002" => Ok(AcadVersion::Version_2_5),
            "AC1003" => Ok(AcadVersion::Version_2_6),
            "AC1004" => Ok(AcadVersion::R9),
            "AC1006" => Ok(AcadVersion::R10),
            "AC1009" => Ok(AcadVersion::R12),
            "AC1011" => Ok(AcadVersion::R13),
            "AC1012" => Ok(AcadVersion::R13),
            "AC1014" => Ok(AcadVersion::R14),
            "14" => Ok(AcadVersion::R14),
            "14.01" => Ok(AcadVersion::R14),
            "AC1015" => Ok(AcadVersion::R2000),
            "15.0" => Ok(AcadVersion::R2000),
            "15.05" => Ok(AcadVersion::R2000),
            "15.06" => Ok(AcadVersion::R2000),
            "AC1018" => Ok(AcadVersion::R2004),
            "16.0" => Ok(AcadVersion::R2004),
            "16.1" => Ok(AcadVersion::R2004),
            "16.2" => Ok(AcadVersion::R2004),
            "AC1021" => Ok(AcadVersion::R2007),
            "17.0" => Ok(AcadVersion::R2007),
            "17.1" => Ok(AcadVersion::R2007),
            "17.2" => Ok(AcadVersion::R2007),
            "AC1024" => Ok(AcadVersion::R2010),
            "18.0" => Ok(AcadVersion::R2010),
            "18.1" => Ok(AcadVersion::R2010),
            "18.2" => Ok(AcadVersion::R2010),
            "AC1027" => Ok(AcadVersion::R2013),
            "19.0" => Ok(AcadVersion::R2013),
            "19.1" => Ok(AcadVersion::R2013),
            "19.2" => Ok(AcadVersion::R2013),
            "19.3" => Ok(AcadVersion::R2013),
            "AC1032" => Ok(AcadVersion::R2018),
            _ => Err(DxfError::UnexpectedEnumValue(0)), // offset doesn't matter here because this failure might not come from parsing
        }
    }
    pub(crate) fn from_safe(val: String) -> AcadVersion {
        match AcadVersion::from(val) {
            Ok(version) => version,
            _ => AcadVersion::R12, // default to R12
        }
    }
}

impl fmt::Display for AcadVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_value = match self {
            AcadVersion::Version_1_0 => "MC0.0",
            AcadVersion::Version_1_2 => "AC1.2",
            AcadVersion::Version_1_40 => "AC1.40",
            AcadVersion::Version_2_05 => "AC1.50",
            AcadVersion::Version_2_10 => "AC2.10",
            AcadVersion::Version_2_21 => "AC2.21",
            AcadVersion::Version_2_22 => "AC2.22",
            AcadVersion::Version_2_5 => "AC1002",
            AcadVersion::Version_2_6 => "AC1003",
            AcadVersion::R9 => "AC1004",
            AcadVersion::R10 => "AC1006",
            AcadVersion::R11 => "AC1009",
            AcadVersion::R12 => "AC1009",
            AcadVersion::R13 => "AC1012",
            AcadVersion::R14 => "AC1014",
            AcadVersion::R2000 => "AC1015",
            AcadVersion::R2004 => "AC1018",
            AcadVersion::R2007 => "AC1021",
            AcadVersion::R2010 => "AC1024",
            AcadVersion::R2013 => "AC1027",
            AcadVersion::R2018 => "AC1032",
        };
        write!(f, "{}", display_value)
    }
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum AngleDirection {
    CounterClockwise = 0,
    Clockwise = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum AngleFormat {
    DecimalDegrees = 0,
    DegreesMinutesSeconds = 1,
    Gradians = 2,
    Radians = 3,
    SurveyorsUnits = 4,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum AttachmentPoint {
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
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum AttributeVisibility {
    None = 0,
    Normal = 1,
    All = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum BackgroundFillSetting {
    Off = 0,
    UseBackgroundFillColor = 1,
    UseDrawingWindowColor = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum BottomTextAttachmentDirection {
    Center = 9,
    UnderlineAndCenter = 10,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum CoordinateDisplay {
    Static = 0,
    ContinuousUpdate = 1,
    DistanceAngleFormat = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DefaultLightingType
{
    OneDistantLight = 0,
    TwoDistantLights = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DesignCoordinateType
{
    Unknown = 0,
    LocalGrid = 1,
    ProjectedGrid = 2,
    Geographic = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DiagnosticBSPMode
{
    Depth = 0,
    Size = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DiagnosticPhotonMode
{
    Density = 0,
    Irradiance = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DictionaryDuplicateRecordHandling
{
    NotApplicable = 0,
    KeepExisting = 1,
    UseClone = 2,
    UpdateXrefAndName = 3,
    UpdateName = 4,
    UnmangleName = 5,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DimensionArcSymbolDisplayMode {
    SymbolBeforeText = 0,
    SymbolAboveText = 1,
    Suppress = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DimensionAssociativity {
    NoAssociationExploded = 0,
    NonAssociativeObjects = 1,
    AssociativeObjects = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DimensionFit {
    TextAndArrowsOutsideLines = 0,
    MoveArrowsFirst = 1,
    MoveTextFirst = 2,
    MoveEitherForBestFit = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DimensionFractionFormat {
    HorizontalStacking = 0,
    DiagonalStacking = 1,
    NotStacked = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DimensionTextBackgroundColorMode {
    None = 0,
    UseDrawingBackground = 1,
    Custom = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DimensionTextJustification {
    AboveLineCenter = 0,
    AboveLineNextToFirstExtension = 1,
    AboveLineNextToSecondExtension = 2,
    AboveLineCenteredOnFirstExtension = 3,
    AboveLineCenteredOnSecondExtension = 4,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DimensionTextMovementRule {
    MoveLineWithText = 0,
    AddLeaderWhenTextIsMoved = 1,
    MoveTextFreely = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DimensionType {
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
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DragMode {
    Off = 0,
    On = 1,
    Auto = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DrawingDirection {
    LeftToRight = 1,
    TopToBottom = 3,
    ByStyle = 5,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DrawingUnits {
    English = 0,
    Metric = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum EdgeStyleModel
{
    NoEdges = 0,
    IsoLines = 1,
    FacetEdges = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum EndCapSetting {
    None = 0,
    Round = 1,
    Angle = 2,
    Square = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FaceColorMode
{
    NoColor = 0,
    ObjectColor = 1,
    BackgroundColor = 2,
    CustomColor = 3,
    MonoColor = 4,
    Tinted = 5,
    Desaturated = 6,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FaceLightingModel
{
    Invisible = 0,
    Visible = 1,
    Phong = 2,
    Gooch = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FaceLightingQuality
{
    None = 0,
    PerFace = 1,
    PerVertex = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FaceModifier
{
    None = 0,
    Opacity = 1,
    Specular = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FlowDirection
{
    Down = 0,
    Up = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FontType {
    TTF = 0,
    SHX = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum GeoDataVersion
{
    R2009 = 1,
    R2010 = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum HelixConstraint {
    ConstrainTurnHeight = 0,
    ConstrainTurns = 1,
    ConstrainHeight = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum HorizontalTextJustification {
    Left = 0,
    Center = 1,
    Right = 2,
    Aligned = 3,
    Middle = 4,
    Fit = 5,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum ImageClippingBoundaryType {
    Rectangular = 1,
    Polygonal = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum ImageResolutionUnits
{
    NoUnits = 0,
    Centimeters = 2,
    Inches = 5,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum JoinStyle {
    None = 0,
    Round = 1,
    Angle = 2,
    Flat = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum Justification {
    Top = 0,
    Middle = 1,
    Bottom = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum LayerAndSpatialIndexSaveMode {
    None = 0,
    LayerIndex = 1,
    SpatialIndex = 2,
    LayerAndSpatialIndex = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum LeaderCreationAnnotationType {
    WithTextAnnotation = 0,
    WithToleranceAnnotation = 1,
    WithBlockReferenceAnnotation = 2,
    NoAnnotation = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum LeaderHooklineDirection {
    OppositeFromHorizontalVector = 0,
    SameAsHorizontalVector = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum LeaderPathType {
    StraightLineSegments = 0,
    Spline = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum LightAttenuationType {
    None = 0,
    InverseLinear = 1,
    InverseSquare = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum LightType {
    Distant = 1,
    Point = 2,
    Spot = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum LineTypeStyle {
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
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum LoftedObjectNormalMode {
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
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum MapAutoTransformMethod
{
    NoAutoTransform = 1,
    ScaleToCurrentEntity = 2,
    IncludeCurrentBlockTransform = 4,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum MapProjectionMethod
{
    Planar = 1,
    Box = 2,
    Cylinder = 3,
    Sphere = 4,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum MapTilingMethod
{
    Tile = 1,
    Crop = 2,
    Clamp = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum MTextFlag {
    MultilineAttribute = 2,
    ConstantMultilineAttributeDefinition = 4,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum MTextLineSpacingStyle {
    AtLeast = 1,
    Exact = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum NonAngularUnits {
    Scientific = 1,
    Decimal = 2,
    Engineering = 3,
    Architectural = 4,
    Fractional = 5,
    WindowsDesktop = 6,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum ObjectOsnapType
{
    None = 0,
    Endpoint = 1,
    Midpoint = 2,
    Center = 3,
    Node = 4,
    Quadrant = 5,
    Intersection = 6,
    Insertion = 7,
    Perpendicular = 8,
    Tangent = 9,
    Nearest = 10,
    ApparentIntersection = 11,
    Parallel = 12,
    StartPoint = 13,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum OleObjectType {
    Link = 1,
    Embedded = 2,
    Static = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum OrthographicViewType {
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
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum PickStyle {
    None = 0,
    Group = 1,
    AssociativeHatch = 2,
    GroupAndAssociativeHatch = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum PlotPaperUnits
{
    Inches = 0,
    Millimeters = 1,
    Pixels = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum PlotRotation
{
    NoRotation = 0,
    CounterClockwise90Degrees = 1,
    UpsideDown = 2,
    Clockwise90Degrees = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum PlotStyle {
    ByLayer = 0,
    ByBlock = 1,
    ByDictionaryDefault = 2,
    ByObjectId = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum PlotType
{
    LastScreenDisplay = 0,
    DrawingExtents = 1,
    DrawingLimits = 2,
    SpecificView = 3,
    SpecificWindow = 4,
    LayoutInformation = 5,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum PolylineCurvedAndSmoothSurfaceType {
    None = 0,
    QuadraticBSpline = 5,
    CubicBSpline = 6,
    Bezier = 8,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum PolySketchMode {
    SketchLines = 0,
    SketchPolylines = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RasterImageUnits
{
    None = 0,
    Millimeter = 1,
    Centimeter = 2,
    Meter = 3,
    Kilometer = 4,
    Inch = 5,
    Foot = 6,
    Yard = 7,
    Mile = 8,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RenderAccuracy
{
    Low = 0,
    Draft = 1,
    High = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RenderDestination
{
    RenderWindow = 0,
    Viewport = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RenderDiagnosticGridMode
{
    Object = 0,
    World = 1,
    Camera = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RenderDiagnosticMode
{
    Off = 0,
    Grid = 1,
    Photon = 2,
    BSP = 4,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RenderDuration
{
    ByTime = 0,
    ByLevel = 1,
    UntilSatisfactory = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RenderProcedure
{
    View = 0,
    Crop = 1,
    Selection = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RenderShadowMode
{
    Simple = 0,
    Sort = 1,
    Segment = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RotatedDimensionType
{
    Parallel = 0,
    Perpendicular = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum SamplingFilterType
{
    Box = 0,
    Triangle = 1,
    Gauss = 2,
    Mitchell = 3,
    Lanczos = 4,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum ScaleEstimationMethod
{
    None = 1,
    UserSpecified = 2,
    GridAtReferencePoint = 3,
    Prismoidal = 4,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum ShadeEdgeMode {
    FacesShadedEdgeNotHighlighted = 0,
    FacesShadedEdgesHighlightedInBlack = 1,
    FacesNotFilledEdgesInEntityColor = 2,
    FacesInEntityColorEdgesInBlack = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum ShadePlotMode
{
    AsDisplayed = 0,
    Wireframe = 1,
    Hidden = 2,
    Rendered = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum ShadePlotResolutionLevel
{
    Draft = 0,
    Preview = 1,
    Normal = 2,
    Presentation = 3,
    Maximum = 4,
    Custom = 5,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum ShadowMode {
    CastsAndReceivesShadows = 0,
    CastsShadows = 1,
    ReceivesShadows = 2,
    IgnoresShadows = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum ShadowType {
    RayTraced = 0,
    ShadowMaps = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum SnapIsometricPlane {
    Left = 0,
    Top = 1,
    Right = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum SnapStyle {
    Standard = 0,
    Isometric = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum SolidHistoryMode {
    None = 0,
    DoesNotOverride = 1,
    Override = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
pub enum StandardScale
{
    ScaledToFit = 0,
    Scale_1_128_inch_to_1_foot = 1,
    Scale_1_64_inch_to_1_foot = 2,
    Scale_1_32_inch_to_1_foot = 3,
    Scale_1_16_inch_to_1_foot = 4,
    Scale_3_32_inch_to_1_foot = 5,
    Scale_1_8_inch_to_1_foot = 6,
    Scale_3_16_inch_to_1_foot = 7,
    Scale_1_4_inch_to_1_foot = 8,
    Scale_3_8_inch_to_1_foot = 9,
    Scale_1_2_inch_to_1_foot = 10,
    Scale_3_4_inch_to_1_foot = 11,
    Scale_1_inch_to_1_foot = 12,
    Scale_3_inches_to_1_foot = 13,
    Scale_6_inches_to_1_foot = 14,
    Scale_1_foot_to_1_foot = 15,
    Scale_1_to_1 = 16,
    Scale_1_to_2 = 17,
    Scale_1_to_4 = 18,
    Scale_1_to_8 = 19,
    Scale_1_to_10 = 20,
    Scale_1_to_16 = 21,
    Scale_1_to_20 = 22,
    Scale_1_to_30 = 23,
    Scale_1_to_40 = 24,
    Scale_1_to_50 = 25,
    Scale_1_to_100 = 26,
    Scale_2_to_1 = 27,
    Scale_4_to_1 = 28,
    Scale_8_to_1 = 29,
    Scale_10_to_1 = 30,
    Scale_100_to_1 = 31,
    Scale_1000_to_1 = 32,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum SubentityType
{
    Edge = 1,
    Face = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum TextAttachmentDirection {
    Horizontal = 0,
    Vertical = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum TextDirection {
    LeftToRight = 0,
    RightToLeft = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum TextLineSpacingStyle {
    AtLeast = 1,
    Exact = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum TileModeDescriptor {
    InTiledViewport = 0,
    InNonTiledViewport = 1,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum TileOrder
{
    Hilbert = 0,
    Spiral = 1,
    LeftToRight = 2,
    RightToLeft = 3,
    TopToBottom = 4,
    BottomToTop = 5,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum UcsOrthographicType
{
    NotOrthographic = 0,
    Top = 1,
    Bottom = 2,
    Front = 3,
    Back = 4,
    Left = 5,
    Right = 6,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
pub enum DrawingTimeZone {
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
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum TopTextAttachmentDirection {
    Center = 9,
    OverlineAndCenter = 10,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum UnderlayFrameMode {
    None = 0,
    DisplayAndPlot = 1,
    DisplayNoPlot = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum UnitFormat {
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
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum Units {
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
    USSurveyFeet = 21,
    USSurveyInch = 22,
    USSurveyYard = 23,
    USSurveyMile = 24,
}
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct ViewMode {
    flags: i32,
}

impl ViewMode {
    pub fn from_i16(val: i16) -> Self {
        ViewMode {
            flags: i32::from(val),
        }
    }
    pub fn raw(self) -> i32 {
        self.flags
    }
    fn flag(self, mask: i32) -> bool {
        self.flags & mask != 0
    }
    fn set_flag(&mut self, mask: i32, val: bool) {
        if val {
            self.flags |= mask;
        } else {
            self.flags &= !mask
        }
    }
    pub fn is_perspective_view_active(self) -> bool {
        self.flag(1)
    }
    pub fn set_is_perspective_view_active(&mut self, val: bool) {
        self.set_flag(1, val)
    }
    pub fn is_front_clipping_on(self) -> bool {
        self.flag(2)
    }
    pub fn set_is_front_clipping_on(&mut self, val: bool) {
        self.set_flag(2, val)
    }
    pub fn is_back_clipping_on(self) -> bool {
        self.flag(4)
    }
    pub fn set_is_back_clipping_on(&mut self, val: bool) {
        self.set_flag(4, val)
    }
    pub fn is_ucs_follow_mode_on(self) -> bool {
        self.flag(8)
    }
    pub fn set_is_ucs_follow_mode_on(&mut self, val: bool) {
        self.set_flag(8, val)
    }
    pub fn is_front_clipping_at_eye(self) -> bool {
        self.flag(16)
    }
    pub fn set_is_front_clipping_at_eye(&mut self, val: bool) {
        self.set_flag(16, val)
    }
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum ViewRenderMode
{
    Classic2D = 0,
    Wireframe = 1,
    HiddenLine = 2,
    FlatShaded = 3,
    GouraudShaded = 4,
    FlatShadedWithWireframe = 5,
    GouraudShadedWithWireframe = 6,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum Version {
    R2010 = 0,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum VerticalTextJustification {
    Baseline = 0,
    Bottom = 1,
    Middle = 2,
    Top = 3,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum XrefClippingBoundaryVisibility {
    NotDisplayedNotPlotted = 0,
    DisplayedAndPlotted = 1,
    DisplayedNotPlotted = 2,
}
}

enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum UnitZeroSuppression {
    SuppressZeroFeetAndZeroInches = 0,
    IncludeZeroFeetAndZeroInches = 1,
    IncludeZeroFeetAndSuppressZeroInches = 2,
    IncludeZeroInchesAndSuppressZeroFeet = 3,
}
}
