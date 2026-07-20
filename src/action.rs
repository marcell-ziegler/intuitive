#[derive(Debug, PartialEq)]
pub enum Action {
    // navigation / turn tracking
    SelectNextRow,
    SelectPreviousRow,
    AdvanceTurn,
    SwitchPanel,

    // Editor lifecycle
    OpenEditorAtIndex(u16), // Index to edit at
    OpenEditorWithNewCreature,
    CloseEditor,
    EditorNextField,
    EditorPrevField,
    SubmitEditor,
    EditorInput(crossterm::event::Event), // raw event — see note

    Quit,
}
