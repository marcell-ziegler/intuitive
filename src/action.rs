pub enum Action {
    // navigation / turn tracking
    SelectNextRow,
    SelectPreviousRow,
    AdvanceTurn,
    SwitchPanel,

    // Editor lifecycle
    OpenEditor,
    CloseEditor,
    EditorNextField,
    EditorPrevField,
    SubmitEditor,
    EditorInput(crossterm::event::Event), // raw event — see note

    Quit,
}
