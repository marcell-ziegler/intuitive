#[derive(Debug, PartialEq)]
pub enum Action {
    // navigation / turn tracking
    SelectNextRow,
    SelectPreviousRow,
    AdvanceTurn,
    SwitchPanel,

    // Editor lifecycle
    OpenEditorAtIndex(usize),
    OpenEditorWithNewCreature,
    CloseEditor,
    EditorNextField,
    EditorPrevField,
    EditorToggleCreatureType,
    SubmitEditor,
    EditorInput(crossterm::event::Event), // raw event — see note

    // Creature  handling
    DeleteCreatureAtIndex(usize),
    DamageCreatureAtIndex(usize),

    Quit,
}
