%%
%% IEEEtran.bst
%% BibTeX Bibliography Style file for IEEE Journals and Conferences (unsorted)
%% Version 1.12 (2007/01/11)
%% 
%% Copyright (c) 2003-2007 Michael Shell
%% 
%% Original starting code base and algorithms obtained from the output of
%% Patrick W. Daly's makebst package as well as from prior versions of
%% IEEE BibTeX styles:
%% 
%% 1. Howard Trickey and Oren Patashnik's ieeetr.bst  (1985/1988)
%% 2. Silvano Balemi and Richard H. Roy's IEEEbib.bst (1993)
%% 
%% Support sites:
%% http://www.michaelshell.org/tex/ieeetran/
%% http://www.ctan.org/tex-archive/macros/latex/contrib/IEEEtran/
%% and/or
%% http://www.ieee.org/
%% 
%% For use with BibTeX version 0.99a or later
%%
%% This is a numerical citation style.
%% 
%%*************************************************************************
%% Legal Notice:
%% This code is offered as-is without any warranty either expressed or
%% implied; without even the implied warranty of MERCHANTABILITY or
%% FITNESS FOR A PARTICULAR PURPOSE! 
%% User assumes all risk.
%% In no event shall IEEE or any contributor to this code be liable for
%% any damages or losses, including, but not limited to, incidental,
%% consequential, or any other damages, resulting from the use or misuse
%% of any information contained here.
%%
%% All comments are the opinions of their respective authors and are not
%% necessarily endorsed by the IEEE.
%%
%% This work is distributed under the LaTeX Project Public License (LPPL)
%% ( http://www.latex-project.org/ ) version 1.3, and may be freely used,
%% distributed and modified. A copy of the LPPL, version 1.3, is included
%% in the base LaTeX documentation of all distributions of LaTeX released
%% 2003/12/01 or later.
%% Retain all contribution notices and credits.
%% ** Modified files should be clearly indicated as such, including  **
%% ** renaming them and changing author support contact information. **
%%
%% File list of work: IEEEabrv.bib, IEEEfull.bib, IEEEexample.bib,
%%                    IEEEtran.bst, IEEEtranS.bst, IEEEtranSA.bst,
%%                    IEEEtranN.bst, IEEEtranSN.bst, IEEEtran_bst_HOWTO.pdf
%%*************************************************************************
%
%
% Changelog:
%
% 1.00 (2002/08/13) Initial release
%
% 1.10 (2002/09/27)
%  1. Corrected minor bug for improperly formed warning message when a
%     book was not given a title. Thanks to Ming Kin Lai for reporting this.
%  2. Added support for CTLname_format_string and CTLname_latex_cmd fields
%     in the BST control entry type.
%
% 1.11 (2003/04/02)
%  1. Fixed bug with URLs containing underscores when using url.sty. Thanks
%     to Ming Kin Lai for reporting this.
%
% 1.12 (2007/01/11)
%  1. Fixed bug with unwanted comma before "et al." when an entry contained
%     more than two author names. Thanks to Pallav Gupta for reporting this.
%  2. Fixed bug with anomalous closing quote in tech reports that have a
%     type, but without a number or address. Thanks to Mehrdad Mirreza for
%     reporting this.
%  3. Use braces in \providecommand in begin.bib to better support
%     latex2html. TeX style length assignments OK with recent versions
%     of latex2html - 1.71 (2002/2/1) or later is strongly recommended.
%     Use of the language field still causes trouble with latex2html.
%     Thanks to Federico Beffa for reporting this.
%  4. Added IEEEtran.bst ID and version comment string to .bbl output.
%  5. Provide a \BIBdecl hook that allows the user to execute commands
%     just prior to the first entry.
%  6. Use default urlstyle (is using url.sty) of "same" rather than rm to
%     better work with a wider variety of bibliography styles.
%  7. Changed month abbreviations from Sept., July and June to Sep., Jul.,
%     and Jun., respectively, as IEEE now does. Thanks to Moritz Borgmann
%     for reporting this.
%  8. Control entry types should not be considered when calculating longest
%     label width.
%  9. Added alias www for electronic/online.
% 10. Added CTLname_url_prefix control entry type.


%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% DEFAULTS FOR THE CONTROLS OF THE BST STYLE %%
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

% These are the defaults for the user adjustable controls. The values used
% here can be overridden by the user via IEEEtranBSTCTL entry type.

% NOTE: The recommended LaTeX command to invoke a control entry type is:
% 
%\makeatletter
%\def\bstctlcite{\@ifnextchar[{\@bstctlcite}{\@bstctlcite[@auxout]}}
%\def\@bstctlcite[#1]#2{\@bsphack
%  \@for\@citeb:=#2\do{%
%    \edef\@citeb{\expandafter\@firstofone\@citeb}%
%    \if@filesw\immediate\write\csname #1\endcsname{\string\citation{\@citeb}}\fi}%
%  \@esphack}
%\makeatother
%
% It is called at the start of the document, before the first \cite, like:
% \bstctlcite{IEEEexample:BSTcontrol}
%
% IEEEtran.cls V1.6 and later does provide this command.



% #0 turns off the display of the number for articles.
% #1 enables
FUNCTION {default.is.use.number.for.article} { #1 }


% #0 turns off the display of the paper and type fields in @inproceedings.
% #1 enables
FUNCTION {default.is.use.paper} { #1 }


% #0 turns off the forced use of "et al."
% #1 enables
FUNCTION {default.is.forced.et.al} { #0 }

% The maximum number of names that can be present beyond which an "et al."
% usage is forced. Be sure that num.names.shown.with.forced.et.al (below)
% is not greater than this value!
% Note: There are many instances of references in IEEE journals which have
% a very large number of authors as well as instances in which "et al." is
% used profusely.
FUNCTION {default.max.num.names.before.forced.et.al} { #10 }

% The number of names that will be shown with a forced "et al.".
% Must be less than or equal to max.num.names.before.forced.et.al
FUNCTION {default.num.names.shown.with.forced.et.al} { #1 }


% #0 turns off the alternate interword spacing for entries with URLs.
% #1 enables
FUNCTION {default.is.use.alt.interword.spacing} { #1 }

% If alternate interword spacing for entries with URLs is enabled, this is
% the interword spacing stretch factor that will be used. For example, the
% default "4" here means that the interword spacing in entries with URLs can
% stretch to four times normal. Does not have to be an integer. Note that
% the value specified here can be overridden by the user in their LaTeX
% code via a command such as: 
% "\providecommand\BIBentryALTinterwordstretchfactor{1.5}" in addition to
% that via the IEEEtranBSTCTL entry type.
FUNCTION {default.ALTinterwordstretchfactor} { "4" }


% #0 turns off the "dashification" of repeated (i.e., identical to those
% of the previous entry) names. IEEE normally does this.
% #1 enables
FUNCTION {default.is.dash.repeated.names} { #1 }


% The default name format control string.
FUNCTION {default.name.format.string}{ "{f.~}{vv~}{ll}{, jj}" }


% The default LaTeX font command for the names.
FUNCTION {default.name.latex.cmd}{ "" }


% The default URL prefix.
FUNCTION {default.name.url.prefix}{ "[Online]. Available:" }


% Other controls that cannot be accessed via IEEEtranBSTCTL entry type.

% #0 turns off the terminal startup banner/completed message so as to
% operate more quietly.
% #1 enables
FUNCTION {is.print.banners.to.terminal} { #1 }




%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% FILE VERSION AND BANNER %%
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

FUNCTION{bst.file.version} { "1.12" }
FUNCTION{bst.file.date} { "2007/01/11" }
FUNCTION{bst.file.website} { "http://www.michaelshell.org/tex/ieeetran/bibtex/" }

FUNCTION {banner.message}
{ is.print.banners.to.terminal
     { "-- IEEEtran.bst version" " " * bst.file.version *
       " (" * bst.file.date * ") " * "by Michael Shell." *
       top$
       "-- " bst.file.website *
       top$
       "-- See the " quote$ * "IEEEtran_bst_HOWTO.pdf" * quote$ * " manual for usage information." *
       top$
     }
     { skip$ }
   if$
}

FUNCTION {completed.message}
{ is.print.banners.to.terminal
     { ""
       top$
       "Done."
       top$
     }
     { skip$ }
   if$
}




%%%%%%%%%%%%%%%%%%%%%%
%% STRING CONSTANTS %%
%%%%%%%%%%%%%%%%%%%%%%

FUNCTION {bbl.and}{ "and" }
FUNCTION {bbl.etal}{ "et~al." }
FUNCTION {bbl.editors}{ "eds." }
FUNCTION {bbl.editor}{ "ed." }
FUNCTION {bbl.edition}{ "ed." }
FUNCTION {bbl.volume}{ "vol." }
FUNCTION {bbl.of}{ "of" }
FUNCTION {bbl.number}{ "no." }
FUNCTION {bbl.in}{ "in" }
FUNCTION {bbl.pages}{ "pp." }
FUNCTION {bbl.page}{ "p." }
FUNCTION {bbl.chapter}{ "ch." }
FUNCTION {bbl.paper}{ "paper" }
FUNCTION {bbl.part}{ "pt." }
FUNCTION {bbl.patent}{ "Patent" }
FUNCTION {bbl.patentUS}{ "U.S." }
FUNCTION {bbl.revision}{ "Rev." }
FUNCTION {bbl.series}{ "ser." }
FUNCTION {bbl.standard}{ "Std." }
FUNCTION {bbl.techrep}{ "Tech. Rep." }
FUNCTION {bbl.mthesis}{ "Master's thesis" }
FUNCTION {bbl.phdthesis}{ "Ph.D. dissertation" }
FUNCTION {bbl.st}{ "st" }
FUNCTION {bbl.nd}{ "nd" }
FUNCTION {bbl.rd}{ "rd" }
FUNCTION {bbl.th}{ "th" }


% This is the LaTeX spacer that is used when a larger than normal space
% is called for (such as just before the address:publisher).
FUNCTION {large.space} { "\hskip 1em plus 0.5em minus 0.4em\relax " }

% The LaTeX code for dashes that are used to represent repeated names.
% Note: Some older IEEE journals used something like
% "\rule{0.275in}{0.5pt}\," which is fairly thick and runs right along
% the baseline. However, IEEE now uses a thinner, above baseline,
% six dash long sequence.
FUNCTION {repeated.name.dashes} { "------" }



%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% PREDEFINED STRING MACROS %%
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

MACRO {jan} {"Jan."}
MACRO {feb} {"Feb."}
MACRO {mar} {"Mar."}
MACRO {apr} {"Apr."}
MACRO {may} {"May"}
MACRO {jun} {"Jun."}
MACRO {jul} {"Jul."}
MACRO {aug} {"Aug."}
MACRO {sep} {"Sep."}
MACRO {oct} {"Oct."}
MACRO {nov} {"Nov."}
MACRO {dec} {"Dec."}



%%%%%%%%%%%%%%%%%%
%% ENTRY FIELDS %%
%%%%%%%%%%%%%%%%%%

ENTRY
  { address
    assignee
    author
    booktitle
    chapter
    day
    dayfiled
    edition
    editor
    howpublished
    institution
    intype
    journal
    key
    language
    month
    monthfiled
    nationality
    note
    number
    organization
    pages
    paper
    publisher
    school
    series
    revision
    title
    type
    url
    volume
    year
    yearfiled
    CTLuse_article_number
    CTLuse_paper
    CTLuse_forced_etal
    CTLmax_names_forced_etal
    CTLnames_show_etal
    CTLuse_alt_spacing
    CTLalt_stretch_factor
    CTLdash_repeated_names
    CTLname_format_string
    CTLname_latex_cmd
    CTLname_url_prefix
  }
  {}
  { label }




%%%%%%%%%%%%%%%%%%%%%%%
%% INTEGER VARIABLES %%
%%%%%%%%%%%%%%%%%%%%%%%

INTEGERS { prev.status.punct this.status.punct punct.std
           punct.no punct.comma punct.period 
           prev.status.space this.status.space space.std
           space.no space.normal space.large
           prev.status.quote this.status.quote quote.std
           quote.no quote.close
           prev.status.nline this.status.nline nline.std
           nline.no nline.newblock 
           status.cap cap.std
           cap.no cap.yes}

INTEGERS { longest.label.width multiresult nameptr namesleft number.label numnames }

INTEGERS { is.use.number.for.article
           is.use.paper
           is.forced.et.al
           max.num.names.before.forced.et.al
           num.names.shown.with.forced.et.al
           is.use.alt.interword.spacing
           is.dash.repeated.names}


%%%%%%%%%%%%%%%%%%%%%%
%% STRING VARIABLES %%
%%%%%%%%%%%%%%%%%%%%%%

STRINGS { bibinfo
          longest.label
          oldname
          s
          t
          ALTinterwordstretchfactor
          name.format.string
          name.latex.cmd
          name.url.prefix}




%%%%%%%%%%%%%%%%%%%%%%%%%
%% LOW LEVEL FUNCTIONS %%
%%%%%%%%%%%%%%%%%%%%%%%%%

FUNCTION {initialize.controls}
{ default.is.use.number.for.article 'is.use.number.for.article :=
  default.is.use.paper 'is.use.paper :=
  default.is.forced.et.al 'is.forced.et.al :=
  default.max.num.names.before.forced.et.al 'max.num.names.before.forced.et.al :=
  default.num.names.shown.with.forced.et.al 'num.names.shown.with.forced.et.al :=
  default.is.use.alt.interword.spacing 'is.use.alt.interword.spacing :=
  default.is.dash.repeated.names 'is.dash.repeated.names :=
  default.ALTinterwordstretchfactor 'ALTinterwordstretchfactor :=
  default.name.format.string 'name.format.string :=
  default.name.latex.cmd 'name.latex.cmd :=
  default.name.url.prefix 'name.url.prefix :=
}


% This IEEEtran.bst features a very powerful and flexible mechanism for
% controlling the capitalization, punctuation, spacing, quotation, and
% newlines of the formatted entry fields. (Note: IEEEtran.bst does not need
% or use the newline/newblock feature, but it has been implemented for
% possible future use.) The output states of IEEEtran.bst consist of
% multiple independent attributes and, as such, can be thought of as being
% vectors, rather than the simple scalar values ("before.all", 
% "mid.sentence", etc.) used in most other .bst files.
% 
% The more flexible and complex design used here was motivated in part by
% IEEE's rather unusual bibliography style. For example, IEEE ends the
% previous field item with a period and large space prior to the publisher
% address; the @electronic entry types use periods as inter-item punctuation
% rather than the commas used by the other entry types; and URLs are never
% followed by periods even though they are the last item in the entry.
% Although it is possible to accommodate these features with the conventional
% output state system, the seemingly endless exceptions make for convoluted,
% unreliable and difficult to maintain code.
%
% IEEEtran.bst's output state system can be easily understood via a simple
% illustration of two most recently formatted entry fields (on the stack):
%
%               CURRENT_ITEM
%               "PREVIOUS_ITEM
%
% which, in this example, is to eventually appear in the bibliography as:
% 
%               "PREVIOUS_ITEM," CURRENT_ITEM
%
% It is the job of the output routine to take the previous item off of the
% stack (while leaving the current item at the top of the stack), apply its
% trailing punctuation (including closing quote marks) and spacing, and then
% to write the result to BibTeX's output buffer:
% 
%               "PREVIOUS_ITEM," 
% 
% Punctuation (and spacing) between items is often determined by both of the
% items rather than just the first one. The presence of quotation marks
% further complicates the situation because, in standard English, trailing
% punctuation marks are supposed to be contained within the quotes.
% 
% IEEEtran.bst maintains two output state (aka "status") vectors which
% correspond to the previous and current (aka "this") items. Each vector
% consists of several independent attributes which track punctuation,
% spacing, quotation, and newlines. Capitalization status is handled by a
% separate scalar because the format routines, not the output routine,
% handle capitalization and, therefore, there is no need to maintain the
% capitalization attribute for both the "previous" and "this" items.
% 
% When a format routine adds a new item, it copies the current output status
% vector to the previous output status vector and (usually) resets the
% current (this) output status vector to a "standard status" vector. Using a
% "standard status" vector in this way allows us to redefine what we mean by
% "standard status" at the start of each entry handler and reuse the same
% format routines under the various inter-item separation schemes. For
% example, the standard status vector for the @book entry type may use
% commas for item separators, while the @electronic type may use periods,
% yet both entry handlers exploit many of the exact same format routines.
% 
% Because format routines have write access to the output status vector of
% the previous item, they can override the punctuation choices of the
% previous format routine! Therefore, it becomes trivial to implement rules
% such as "Always use a period and a large space before the publisher." By
% pushing the generation of the closing quote mark to the output routine, we
% avoid all the problems caused by having to close a quote before having all
% the information required to determine what the punctuation should be.
%
% The IEEEtran.bst output state system can easily be expanded if needed.
% For instance, it is easy to add a "space.tie" attribute value if the
% bibliography rules mandate that two items have to be joined with an
% unbreakable space. 

FUNCTION {initialize.status.constants}
{ #0 'punct.no :=
  #1 'punct.comma :=
  #2 'punct.period :=
  #0 'space.no := 
  #1 'space.normal :=
  #2 'space.large :=
  #0 'quote.no :=
  #1 'quote.close :=
  #0 'cap.no :=
  #1 'cap.yes :=
  #0 'nline.no :=
  #1 'nline.newblock :=
}

FUNCTION {std.status.using.comma}
{ punct.comma 'punct.std :=
  space.normal 'space.std :=
  quote.no 'quote.std :=
  nline.no 'nline.std :=
  cap.no 'cap.std :=
}

FUNCTION {std.status.using.period}
{ punct.period 'punct.std :=
  space.normal 'space.std :=
  quote.no 'quote.std :=
  nline.no 'nline.std :=
  cap.yes 'cap.std :=
}

FUNCTION {initialize.prev.this.status}
{ punct.no 'prev.status.punct :=
  space.no 'prev.status.space :=
  quote.no 'prev.status.quote :=
  nline.no 'prev.status.nline :=
  punct.no 'this.status.punct :=
  space.no 'this.status.space :=
  quote.no 'this.status.quote :=
  nline.no 'this.status.nline :=
  cap.yes 'status.cap :=
}

FUNCTION {this.status.std}
{ punct.std 'this.status.punct :=
  space.std 'this.status.space :=
  quote.std 'this.status.quote :=
  nline.std 'this.status.nline :=
}

FUNCTION {cap.status.std}{ cap.std 'status.cap := }

FUNCTION {this.to.prev.status}
{ this.status.punct 'prev.status.punct :=
  this.status.space 'prev.status.space :=
  this.status.quote 'prev.status.quote :=
  this.status.nline 'prev.status.nline :=
}


FUNCTION {not}
{   { #0 }
    { #1 }
  if$
}

FUNCTION {and}
{   { skip$ }
    { pop$ #0 }
  if$
}

FUNCTION {or}
{   { pop$ #1 }
    { skip$ }
  if$
}


% convert the strings "yes" or "no" to #1 or #0 respectively
FUNCTION {yes.no.to.int}
{ "l" change.case$ duplicate$
    "yes" =
    { pop$  #1 }
    { duplicate$ "no" =
        { pop$ #0 }
        { "unknown boolean " quote$ * swap$ * quote$ *
          " in " * cite$ * warning$
          #0
        }
      if$
    }
  if$
}


% pushes true if the single char string on the stack is in the
% range of "0" to "9"
FUNCTION {is.num}
{ chr.to.int$
  duplicate$ "0" chr.to.int$ < not
  swap$ "9" chr.to.int$ > not and
}

% multiplies the integer on the stack by a factor of 10
FUNCTION {bump.int.mag}
{ #0 'multiresult :=
    { duplicate$ #0 > }
    { #1 -
      multiresult #10 +
      'multiresult :=
    }
  while$
pop$
multiresult
}

% converts a single character string on the stack to an integer
FUNCTION {char.to.integer}
{ duplicate$ 
  is.num
    { chr.to.int$ "0" chr.to.int$ - }
    {"noninteger character " quote$ * swap$ * quote$ *
          " in integer field of " * cite$ * warning$
    #0
    }
  if$
}

% converts a string on the stack to an integer
FUNCTION {string.to.integer}
{ duplicate$ text.length$ 'namesleft :=
  #1 'nameptr :=
  #0 'numnames :=
    { nameptr namesleft > not }
    { duplicate$ nameptr #1 substring$
      char.to.integer numnames bump.int.mag +
      'numnames :=
      nameptr #1 +
      'nameptr :=
    }
  while$
pop$
numnames
}




% The output routines write out the *next* to the top (previous) item on the
% stack, adding punctuation and such as needed. Since IEEEtran.bst maintains
% the output status for the top two items on the stack, these output
% routines have to consider the previous output status (which corresponds to
% the item that is being output). Full independent control of punctuation,
% closing quote marks, spacing, and newblock is provided.
% 
% "output.nonnull" does not check for the presence of a previous empty
% item.
% 
% "output" does check for the presence of a previous empty item and will
% remove an empty item rather than outputting it.
% 
% "output.warn" is like "output", but will issue a warning if it detects
% an empty item.

FUNCTION {output.nonnull}
{ swap$
  prev.status.punct punct.comma =
     { "," * }
     { skip$ }
   if$
  prev.status.punct punct.period =
     { add.period$ }
     { skip$ }
   if$ 
  prev.status.quote quote.close =
     { "''" * }
     { skip$ }
   if$
  prev.status.space space.normal =
     { " " * }
     { skip$ }
   if$
  prev.status.space space.large =
     { large.space * }
     { skip$ }
   if$
  write$
  prev.status.nline nline.newblock =
     { newline$ "\newblock " write$ }
     { skip$ }
   if$
}

FUNCTION {output}
{ duplicate$ empty$
    'pop$
    'output.nonnull
  if$
}

FUNCTION {output.warn}
{ 't :=
  duplicate$ empty$
    { pop$ "empty " t * " in " * cite$ * warning$ }
    'output.nonnull
  if$
}

% "fin.entry" is the output routine that handles the last item of the entry
% (which will be on the top of the stack when "fin.entry" is called).

FUNCTION {fin.entry}
{ this.status.punct punct.no =
     { skip$ }
     { add.period$ }
   if$
   this.status.quote quote.close =
     { "''" * }
     { skip$ }
   if$
write$
newline$
}


FUNCTION {is.last.char.not.punct}
{ duplicate$
   "}" * add.period$
   #-1 #1 substring$ "." =
}

FUNCTION {is.multiple.pages}
{ 't :=
  #0 'multiresult :=
    { multiresult not
      t empty$ not
      and
    }
    { t #1 #1 substring$
      duplicate$ "-" =
      swap$ duplicate$ "," =
      swap$ "+" =
      or or
        { #1 'multiresult := }
        { t #2 global.max$ substring$ 't := }
      if$
    }
  while$
  multiresult
}

FUNCTION {capitalize}{ "u" change.case$ "t" change.case$ }

FUNCTION {emphasize}
{ duplicate$ empty$
    { pop$ "" }
    { "\emph{" swap$ * "}" * }
  if$
}

FUNCTION {do.name.latex.cmd}
{ name.latex.cmd
  empty$
    { skip$ }
    { name.latex.cmd "{" * swap$ * "}" * }
  if$
}

% IEEEtran.bst uses its own \BIBforeignlanguage command which directly
% invokes the TeX hyphenation patterns without the need of the Babel
% package. Babel does a lot more than switch hyphenation patterns and
% its loading can cause unintended effects in many class files (such as
% IEEEtran.cls).
FUNCTION {select.language}
{ duplicate$ empty$ 'pop$
    { language empty$ 'skip$
        { "\BIBforeignlanguage{" language * "}{" * swap$ * "}" * }
      if$
    }
  if$
}

FUNCTION {tie.or.space.prefix}
{ duplicate$ text.length$ #3 <
    { "~" }
    { " " }
  if$
  swap$
}

FUNCTION {get.bbl.editor}
{ editor num.names$ #1 > 'bbl.editors 'bbl.editor if$ }

FUNCTION {space.word}{ " " swap$ * " " * }


% Field Conditioners, Converters, Checkers and External Interfaces

FUNCTION {empty.field.to.null.string}
{ duplicate$ empty$
    { pop$ "" }
    { skip$ }
  if$
}

FUNCTION {either.or.check}
{ empty$
    { pop$ }
    { "can't use both " swap$ * " fields in " * cite$ * warning$ }
  if$
}

FUNCTION {empty.entry.warn}
{ author empty$ title empty$ howpublished empty$
  month empty$ year empty$ note empty$ url empty$
  and and and and and and
    { "all relevant fields are empty in " cite$ * warning$ }
    'skip$
  if$
}


% The bibinfo system provides a way for the electronic parsing/acquisition
% of a bibliography's contents as is done by ReVTeX. For example, a field
% could be entered into the bibliography as:
% \bibinfo{volume}{2}
% Only the "2" would show up in the document, but the LaTeX \bibinfo command
% could do additional things with the information. IEEEtran.bst does provide
% a \bibinfo command via "\providecommand{\bibinfo}[2]{#2}". However, it is
% currently not used as the bogus bibinfo functions defined here output the
% entry values directly without the \bibinfo wrapper. The bibinfo functions
% themselves (and the calls to them) are retained for possible future use.
% 
% bibinfo.check avoids acting on missing fields while bibinfo.warn will
% issue a warning message if a missing field is detected. Prior to calling
% the bibinfo functions, the user should push the field value and then its
% name string, in that order.

FUNCTION {bibinfo.check}
{ swap$ duplicate$ missing$
    { pop$ pop$ "" }
    { duplicate$ empty$
        { swap$ pop$ }
        { swap$ pop$ }
      if$
    }
  if$
}

FUNCTION {bibinfo.warn}
{ swap$ duplicate$ missing$
    { swap$ "missing " swap$ * " in " * cite$ * warning$ pop$ "" }
    { duplicate$ empty$
        { swap$ "empty " swap$ * " in " * cite$ * warning$ }
        { swap$ pop$ }
      if$
    }
  if$
}


% IEEE separates large numbers with more than 4 digits into groups of
% three. IEEE uses a small space to separate these number groups. 
% Typical applications include patent and page numbers.

% number of consecutive digits required to trigger the group separation.
FUNCTION {large.number.trigger}{ #5 }

% For numbers longer than the trigger, this is the blocksize of the groups.
% The blocksize must be less than the trigger threshold, and 2 * blocksize
% must be greater than the trigger threshold (can't do more than one
% separation on the initial trigger).
FUNCTION {large.number.blocksize}{ #3 }

% What is actually inserted between the number groups.
FUNCTION {large.number.separator}{ "\," }

% So as to save on integer variables by reusing existing ones, numnames
% holds the current number of consecutive digits read and nameptr holds
% the number that will trigger an inserted space.
FUNCTION {large.number.separate}
{ 't :=
  ""
  #0 'numnames :=
  large.number.trigger 'nameptr :=
  { t empty$ not }
  { t #-1 #1 substring$ is.num
      { numnames #1 + 'numnames := }
      { #0 'numnames := 
        large.number.trigger 'nameptr :=
      }
    if$
    t #-1 #1 substring$ swap$ *
    t #-2 global.max$ substring$ 't :=
    numnames nameptr =
      { duplicate$ #1 nameptr large.number.blocksize - substring$ swap$
        nameptr large.number.blocksize - #1 + global.max$ substring$
        large.number.separator swap$ * *
        nameptr large.number.blocksize - 'numnames :=
        large.number.blocksize #1 + 'nameptr :=
      }
      { skip$ }
    if$
  }
  while$
}

% Converts all single dashes "-" to double dashes "--".
FUNCTION {n.dashify}
{ large.number.separate
  't :=
  ""
    { t empty$ not }
    { t #1 #1 substring$ "-" =
        { t #1 #2 substring$ "--" = not
            { "--" *
              t #2 global.max$ substring$ 't :=
            }
            {   { t #1 #1 substring$ "-" = }
                { "-" *
                  t #2 global.max$ substring$ 't :=
                }
              while$
            }
          if$
        }
        { t #1 #1 substring$ *
          t #2 global.max$ substring$ 't :=
        }
      if$
    }
  while$
}


% This function detects entries with names that are identical to that of
% the previous entry and replaces the repeated names with dashes (if the
% "is.dash.repeated.names" user control is nonzero).
FUNCTION {name.or.dash}
{ 's :=
   oldname empty$
     { s 'oldname := s }
     { s oldname =
         { is.dash.repeated.names
              { repeated.name.dashes }
              { s 'oldname := s }
            if$
         }
         { s 'oldname := s }
       if$
     }
   if$
}

% Converts the number string on the top of the stack to
% "numerical ordinal form" (e.g., "7" to "7th"). There is
% no artificial limit to the upper bound of the numbers as the
% least significant digit always determines the ordinal form.
FUNCTION {num.to.ordinal}
{ duplicate$ #-1 #1 substring$ "1" =
     { bbl.st * }
     { duplicate$ #-1 #1 substring$ "2" =
         { bbl.nd * }
         { duplicate$ #-1 #1 substring$ "3" =
             { bbl.rd * }
             { bbl.th * }
           if$
         }
       if$
     }
   if$
}

% If the string on the top of the stack begins with a number,
% (e.g., 11th) then replace the string with the leading number
% it contains. Otherwise retain the string as-is. s holds the
% extracted number, t holds the part of the string that remains
% to be scanned.
FUNCTION {extract.num}
{ duplicate$ 't :=
  "" 's :=
  { t empty$ not }
  { t #1 #1 substring$
    t #2 global.max$ substring$ 't :=
    duplicate$ is.num
      { s swap$ * 's := }
      { pop$ "" 't := }
    if$
  }
  while$
  s empty$
    'skip$
    { pop$ s }
  if$
}

% Converts the word number string on the top of the stack to
% Arabic string form. Will be successful up to "tenth".
FUNCTION {word.to.num}
{ duplicate$ "l" change.case$ 's :=
  s "first" =
    { pop$ "1" }
    { skip$ }
  if$
  s "second" =
    { pop$ "2" }
    { skip$ }
  if$
  s "third" =
    { pop$ "3" }
    { skip$ }
  if$
  s "fourth" =
    { pop$ "4" }
    { skip$ }
  if$
  s "fifth" =
    { pop$ "5" }
    { skip$ }
  if$
  s "sixth" =
    { pop$ "6" }
    { skip$ }
  if$
  s "seventh" =
    { pop$ "7" }
    { skip$ }
  if$
  s "eighth" =
    { pop$ "8" }
    { skip$ }
  if$
  s "ninth" =
    { pop$ "9" }
    { skip$ }
  if$
  s "tenth" =
    { pop$ "10" }
    { skip$ }
  if$
}


% Converts the string on the top of the stack to numerical
% ordinal (e.g., "11th") form.
FUNCTION {convert.edition}
{ duplicate$ empty$ 'skip$
    { duplicate$ #1 #1 substring$ is.num
        { extract.num
          num.to.ordinal
        }
        { word.to.num
          duplicate$ #1 #1 substring$ is.num
            { num.to.ordinal }
            { "edition ordinal word " quote$ * edition * quote$ *
              " may be too high (or improper) for conversion" * " in " * cite$ * warning$
            }
          if$
        }
      if$
    }
  if$
}




%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
%% LATEX BIBLIOGRAPHY CODE %%
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

FUNCTION {start.entry}
{ newline$
  "\bibitem{" write$
  cite$ write$
  "}" write$
  newline$
  ""
  initialize.prev.this.status
}

% Here we write out all the LaTeX code that we will need. The most involved
% code sequences are those that control the alternate interword spacing and
% foreign language hyphenation patterns. The heavy use of \providecommand
% gives users a way to override the defaults. Special thanks to Javier Bezos,
% Johannes Braams, Robin Fairbairns, Heiko Oberdiek, Donald Arseneau and all
% the other gurus on comp.text.tex for their help and advice on the topic of
% \selectlanguage, Babel and BibTeX.
FUNCTION {begin.bib}
{ "% Generated by IEEEtran.bst, version: " bst.file.version * " (" * bst.file.date * ")" *
  write$ newline$
  preamble$ empty$ 'skip$
    { preamble$ write$ newline$ }
  if$
  "\begin{thebibliography}{"  longest.label  * "}" *
  write$ newline$
  "\providecommand{\url}[1]{#1}"
  write$ newline$
  "\csname url@samestyle\endcsname"
  write$ newline$
  "\providecommand{\newblock}{\relax}"
  write$ newline$
  "\providecommand{\bibinfo}[2]{#2}"
  write$ newline$
  "\providecommand{\BIBentrySTDinterwordspacing}{\spaceskip=0pt\relax}"
  write$ newline$
  "\providecommand{\BIBentryALTinterwordstretchfactor}{"
  ALTinterwordstretchfactor * "}" *
  write$ newline$
  "\providecommand{\BIBentryALTinterwordspacing}{\spaceskip=\fontdimen2\font plus "
  write$ newline$
  "\BIBentryALTinterwordstretchfactor\fontdimen3\font minus \fontdimen4\font\relax}"
  write$ newline$
  "\providecommand{\BIBforeignlanguage}[2]{{%"
  write$ newline$
  "\expandafter\ifx\csname l@#1\endcsname\relax"
  write$ newline$
  "\typeout{** WARNING: IEEEtran.bst: No hyphenation pattern has been}%"
  write$ newline$
  "\typeout{** loaded for the language `#1'. Using the pattern for}%"
  write$ newline$
  "\typeout{** the default language instead.}%"
  write$ newline$
  "\else"
  write$ newline$
  "\language=\csname l@#1\endcsname"
  write$ newline$
  "\fi"
  write$ newline$
  "#2}}"
  write$ newline$
  "\providecommand{\BIBdecl}{\relax}"
  write$ newline$
  "\BIBdecl"
  write$ newline$
}

FUNCTION {end.bib}
{ newline$ "\end{thebibliography}" write$ newline$ }

FUNCTION {if.url.alt.interword.spacing}
{ is.use.alt.interword.spacing 
     {url empty$ 'skip$ {"\BIBentryALTinterwordspacing" write$ newline$} if$}
     { skip$ }
   if$
}

FUNCTION {if.url.std.interword.spacing}
{ is.use.alt.interword.spacing 
     {url empty$ 'skip$ {"\BIBentrySTDinterwordspacing" write$ newline$} if$}
     { skip$ }
   if$
}




%%%%%%%%%%%%%%%%%%%%%%%%
%% LONGEST LABEL PASS %%
%%%%%%%%%%%%%%%%%%%%%%%%

FUNCTION {initialize.longest.label}
{ "" 'longest.label :=
  #1 'number.label :=
  #0 'longest.label.width :=
}

FUNCTION {longest.label.pass}
{ type$ "ieeetranbstctl" =
    { skip$ }
    { number.label int.to.str$ 'label :=
      number.label #1 + 'number.label :=
      label width$ longest.label.width >
        { label 'longest.label :=
          label width$ 'longest.label.width :=
        }
        { skip$ }
      if$
    }
  if$
}




%%%%%%%%%%%%%%%%%%%%%
%% FORMAT HANDLERS %%
%%%%%%%%%%%%%%%%%%%%%

%% Lower Level Formats (used by higher level formats)

FUNCTION {format.address.org.or.pub.date}
{ 't :=
  ""
  year empty$
    { "empty year in " cite$ * warning$ }
    { skip$ }
  if$
  address empty$ t empty$ and
  year empty$ and month empty$ and
    { skip$ }
    { this.to.prev.status
      this.status.std
      cap.status.std
      address "address" bibinfo.check *
      t empty$
        { skip$ }
        { punct.period 'prev.status.punct :=
          space.large 'prev.status.space :=
          address empty$
            { skip$ }
            { ": " * }
          if$
          t *
        }
      if$
      year empty$ month empty$ and
        { skip$ }
        { t empty$ address empty$ and
            { skip$ }
            { ", " * }
          if$
          month empty$
            { year empty$
                { skip$ }
                { year "year" bibinfo.check * }
              if$
            }
            { month "month" bibinfo.check *
              year empty$
                 { skip$ }
                 { " " * year "year" bibinfo.check * }
              if$
            }
          if$
        }
      if$
    }
  if$
}


FUNCTION {format.names}
{ 'bibinfo :=
  duplicate$ empty$ 'skip$ {
  this.to.prev.status
  this.status.std
  's :=
  "" 't :=
  #1 'nameptr :=
  s num.names$ 'numnames :=
  numnames 'namesleft :=
    { namesleft #0 > }
    { s nameptr
      name.format.string
      format.name$
      bibinfo bibinfo.check
      't :=
      nameptr #1 >
        { nameptr num.names.shown.with.forced.et.al #1 + =
          numnames max.num.names.before.forced.et.al >
          is.forced.et.al and and
            { "others" 't :=
              #1 'namesleft :=
            }
            { skip$ }
          if$
          namesleft #1 >
            { ", " * t do.name.latex.cmd * }
            { s nameptr "{ll}" format.name$ duplicate$ "others" =
                { 't := }
                { pop$ }
              if$
              t "others" =
                { " " * bbl.etal emphasize * }
                { numnames #2 >
                    { "," * }
                    { skip$ }
                  if$
                  bbl.and
                  space.word * t do.name.latex.cmd *
                }
              if$
            }
          if$
        }
        { t do.name.latex.cmd }
      if$
      nameptr #1 + 'nameptr :=
      namesleft #1 - 'namesleft :=
    }
  while$
  cap.status.std
  } if$
}




%% Higher Level Formats

%% addresses/locations

FUNCTION {format.address}
{ address duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      cap.status.std
    }
  if$
}



%% author/editor names

FUNCTION {format.authors}{ author "author" format.names }

FUNCTION {format.editors}
{ editor "editor" format.names duplicate$ empty$ 'skip$
    { ", " *
      get.bbl.editor
      capitalize
      *
    }
  if$
}



%% date

FUNCTION {format.date}
{
  month "month" bibinfo.check duplicate$ empty$
  year  "year" bibinfo.check duplicate$ empty$
    { swap$ 'skip$
        { this.to.prev.status
          this.status.std
          cap.status.std
         "there's a month but no year in " cite$ * warning$ }
      if$
      *
    }
    { this.to.prev.status
      this.status.std
      cap.status.std
      swap$ 'skip$
        {
          swap$
          " " * swap$
        }
      if$
      *
    }
  if$
}

FUNCTION {format.date.electronic}
{ month "month" bibinfo.check duplicate$ empty$
  year  "year" bibinfo.check duplicate$ empty$
    { swap$ 
        { pop$ }
        { "there's a month but no year in " cite$ * warning$
        pop$ ")" * "(" swap$ *
        this.to.prev.status
        punct.no 'this.status.punct :=
        space.normal 'this.status.space :=
        quote.no 'this.status.quote :=
        cap.yes  'status.cap :=
        }
      if$
    }
    { swap$ 
        { swap$ pop$ ")" * "(" swap$ * }
        { "(" swap$ * ", " * swap$ * ")" * }
      if$
    this.to.prev.status
    punct.no 'this.status.punct :=
    space.normal 'this.status.space :=
    quote.no 'this.status.quote :=
    cap.yes  'status.cap :=
    }
  if$
}



%% edition/title

% Note: IEEE considers the edition to be closely associated with
% the title of a book. So, in IEEEtran.bst the edition is normally handled 
% within the formatting of the title. The format.edition function is 
% retained here for possible future use.
FUNCTION {format.edition}
{ edition duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      convert.edition
      status.cap
        { "t" }
        { "l" }
      if$ change.case$
      "edition" bibinfo.check
      "~" * bbl.edition *
      cap.status.std
    }
  if$
}

% This is used to format the booktitle of a conference proceedings.
% Here we use the "intype" field to provide the user a way to 
% override the word "in" (e.g., with things like "presented at")
% Use of intype stops the emphasis of the booktitle to indicate that
% we no longer mean the written conference proceedings, but the
% conference itself.
FUNCTION {format.in.booktitle}
{ booktitle "booktitle" bibinfo.check duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      select.language
      intype missing$
        { emphasize
          bbl.in " " *
        }
        { intype " " * }
      if$
      swap$ *
      cap.status.std
    }
  if$
}

% This is used to format the booktitle of collection.
% Here the "intype" field is not supported, but "edition" is.
FUNCTION {format.in.booktitle.edition}
{ booktitle "booktitle" bibinfo.check duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      select.language
      emphasize
      edition empty$ 'skip$
        { ", " *
          edition
          convert.edition
          "l" change.case$
          * "~" * bbl.edition *
        }
      if$
      bbl.in " " * swap$ *
      cap.status.std
    }
  if$
}

FUNCTION {format.article.title}
{ title duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      "t" change.case$
    }
  if$
  "title" bibinfo.check
  duplicate$ empty$ 'skip$
    { quote.close 'this.status.quote :=
      is.last.char.not.punct
        { punct.std 'this.status.punct := }
        { punct.no 'this.status.punct := }
      if$
      select.language
      "``" swap$ *
      cap.status.std
    }
  if$
}

FUNCTION {format.article.title.electronic}
{ title duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      cap.status.std
      "t" change.case$ 
    }
  if$
  "title" bibinfo.check
  duplicate$ empty$ 
    { skip$ } 
    { select.language }
  if$
}

FUNCTION {format.book.title.edition}
{ title "title" bibinfo.check
  duplicate$ empty$
    { "empty title in " cite$ * warning$ }
    { this.to.prev.status
      this.status.std
      select.language
      emphasize
      edition empty$ 'skip$
        { ", " *
          edition
          convert.edition
          status.cap
            { "t" }
            { "l" }
          if$
          change.case$
          * "~" * bbl.edition *
        }
      if$
      cap.status.std
    }
  if$
}

FUNCTION {format.book.title}
{ title "title" bibinfo.check
  duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      cap.status.std
      select.language
      emphasize
    }
  if$
}



%% journal

FUNCTION {format.journal}
{ journal duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      cap.status.std
      select.language
      emphasize
    }
  if$
}



%% how published

FUNCTION {format.howpublished}
{ howpublished duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      cap.status.std
    }
  if$
}



%% institutions/organization/publishers/school

FUNCTION {format.institution}
{ institution duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      cap.status.std
    }
  if$
}

FUNCTION {format.organization}
{ organization duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      cap.status.std
    }
  if$
}

FUNCTION {format.address.publisher.date}
{ publisher "publisher" bibinfo.warn format.address.org.or.pub.date }

FUNCTION {format.address.publisher.date.nowarn}
{ publisher "publisher" bibinfo.check format.address.org.or.pub.date }

FUNCTION {format.address.organization.date}
{ organization "organization" bibinfo.check format.address.org.or.pub.date }

FUNCTION {format.school}
{ school duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      cap.status.std
    }
  if$
}



%% volume/number/series/chapter/pages

FUNCTION {format.volume}
{ volume empty.field.to.null.string
  duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      bbl.volume 
      status.cap
        { capitalize }
        { skip$ }
      if$
      swap$ tie.or.space.prefix
      "volume" bibinfo.check
      * *
      cap.status.std
    }
  if$
}

FUNCTION {format.number}
{ number empty.field.to.null.string
  duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      status.cap
         { bbl.number capitalize }
         { bbl.number }
       if$
      swap$ tie.or.space.prefix
      "number" bibinfo.check
      * *
      cap.status.std
    }
  if$
}

FUNCTION {format.number.if.use.for.article}
{ is.use.number.for.article 
     { format.number }
     { "" }
   if$
}

% IEEE does not seem to tie the series so closely with the volume
% and number as is done in other bibliography styles. Instead the
% series is treated somewhat like an extension of the title.
FUNCTION {format.series}
{ series empty$ 
   { "" }
   { this.to.prev.status
     this.status.std
     bbl.series " " *
     series "series" bibinfo.check *
     cap.status.std
   }
 if$
}


FUNCTION {format.chapter}
{ chapter empty$
    { "" }
    { this.to.prev.status
      this.status.std
      type empty$
        { bbl.chapter }
        { type "l" change.case$
          "type" bibinfo.check
        }
      if$
      chapter tie.or.space.prefix
      "chapter" bibinfo.check
      * *
      cap.status.std
    }
  if$
}


% The intended use of format.paper is for paper numbers of inproceedings.
% The paper type can be overridden via the type field.
% We allow the type to be displayed even if the paper number is absent
% for things like "postdeadline paper"
FUNCTION {format.paper}
{ is.use.paper
     { paper empty$
        { type empty$
            { "" }
            { this.to.prev.status
              this.status.std
              type "type" bibinfo.check
              cap.status.std
            }
          if$
        }
        { this.to.prev.status
          this.status.std
          type empty$
            { bbl.paper }
            { type "type" bibinfo.check }
          if$
          " " * paper
          "paper" bibinfo.check
          *
          cap.status.std
        }
      if$
     }
     { "" } 
   if$
}


FUNCTION {format.pages}
{ pages duplicate$ empty$ 'skip$
    { this.to.prev.status
      this.status.std
      duplicate$ is.multiple.pages
        {
          bbl.pages swap$
          n.dashify
        }
        {
          bbl.page swap$
        }
      if$
      tie.or.space.prefix
      "pages" bibinfo.check
      * *
      cap.status.std
    }
  if$
}



%% technical report number

FUNCTION {format.tech.report.number}
{ number "number" bibinfo.check
  this.to.prev.status
  this.status.std
  cap.status.std
  type duplicate$ empty$
    { pop$ 
      bbl.techrep
    }
    { skip$ }
  if$
  "type" bibinfo.check 
  swap$ duplicate$ empty$
    { pop$ }
    { tie.or.space.prefix * * }
  if$
}



%% note

FUNCTION {format.note}
{ note empty$
    { "" }
    { this.to.prev.status
      this.status.std
      punct.period 'this.status.punct :=
      note #1 #1 substring$
      duplicate$ "{" =
        { skip$ }
        { status.cap
          { "u" }
          { "l" }
        if$
        change.case$
        }
      if$
      note #2 global.max$ substring$ * "note" bibinfo.check
      cap.yes  'status.cap :=
    }
  if$
}



%% patent

FUNCTION {format.patent.date}
{ this.to.prev.status
  this.status.std
  year empty$
    { monthfiled duplicate$ empty$
        { "monthfiled" bibinfo.check pop$ "" }
        { "monthfiled" bibinfo.check }
      if$
      dayfiled duplicate$ empty$
        { "dayfiled" bibinfo.check pop$ "" * }
        { "dayfiled" bibinfo.check 
          monthfiled empty$ 
             { "dayfiled without a monthfiled in " cite$ * warning$
               * 
             }
             { " " swap$ * * }
           if$
        }
      if$
      yearfiled empty$
        { "no year or yearfiled in " cite$ * warning$ }
        { yearfiled "yearfiled" bibinfo.check 
          swap$
          duplicate$ empty$
             { pop$ }
             { ", " * swap$ * }
           if$
        }
      if$
    }
    { month duplicate$ empty$
        { "month" bibinfo.check pop$ "" }
        { "month" bibinfo.check }
      if$
      day duplicate$ empty$
        { "day" bibinfo.check pop$ "" * }
        { "day" bibinfo.check 
          month empty$ 
             { "day without a month in " cite$ * warning$
               * 
             }
             { " " swap$ * * }
           if$
        }
      if$
      year "year" bibinfo.check 
      swap$
      duplicate$ empty$
        { pop$ }
        { ", " * swap$ * }
      if$
    }
  if$
  cap.status.std
}

FUNCTION {format.patent.nationality.type.number}
{ this.to.prev.status
  this.status.std
  nationality duplicate$ empty$
    { "nationality" bibinfo.warn pop$ "" }
    { "nationality" bibinfo.check
      duplicate$ "l" change.case$ "united states" =
        { pop$ bbl.patentUS }
        { skip$ }
      if$
      " " *
    }
  if$
  type empty$
    { bbl.patent "type" bibinfo.check }
    { type "type" bibinfo.check }
  if$  
  *
  number duplicate$ empty$
    { "number" bibinfo.warn pop$ }
    { "number" bibinfo.check
      large.number.separate
      swap$ " " * swap$ *
    }
  if$ 
  cap.status.std
}



%% standard

FUNCTION {format.organization.institution.standard.type.number}
{ this.to.prev.status
  this.status.std
  organization duplicate$ empty$
    { pop$ 
      institution duplicate$ empty$
        { "institution" bibinfo.warn }
        { "institution" bibinfo.warn " " * }
      if$
    }
    { "organization" bibinfo.warn " " * }
  if$
  type empty$
    { bbl.standard "type" bibinfo.check }
    { type "type" bibinfo.check }
  if$  
  *
  number duplicate$ empty$
    { "number" bibinfo.check pop$ }
    { "number" bibinfo.check
      large.number.separate
      swap$ " " * swap$ *
    }
  if$ 
  cap.status.std
}

FUNCTION {format.revision}
{ revision empty$
    { "" }
    { this.to.prev.status
      this.status.std
      bbl.revision
      revision tie.or.space.prefix
      "revision" bibinfo.check
      * *
      cap.status.std
    }
  if$
}


%% thesis

FUNCTION {format.master.thesis.type}
{ this.to.prev.status
  this.status.std
  type empty$
    {
      bbl.mthesis
    }
    { 
      type "type" bibinfo.check
    }
  if$
cap.status.std
}

FUNCTION {format.phd.thesis.type}
{ this.to.prev.status
  this.status.std
  type empty$
    {
      bbl.phdthesis
    }
    { 
      type "type" bibinfo.check
    }
  if$
cap.status.std
}



%% URL

FUNCTION {format.url}
{ url empty$
    { "" }
    { this.to.prev.status
      this.status.std
      cap.yes 'status.cap :=
      name.url.prefix " " *
      "\url{" * url * "}" *
      punct.no 'this.status.punct :=
      punct.period 'prev.status.punct :=
      space.normal 'this.status.space :=
      space.normal 'prev.status.space :=
      quote.no 'this.status.quote :=
    }
  if$
}




%%%%%%%%%%%%%%%%%%%%
%% ENTRY HANDLERS %%
%%%%%%%%%%%%%%%%%%%%


% Note: In many journals, IEEE (or the authors) tend not to show the number
% for articles, so the display of the number is controlled here by the
% switch "is.use.number.for.article"
FUNCTION {article}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors "author" output.warn
  name.or.dash
  format.article.title "title" output.warn
  format.journal "journal" bibinfo.check "journal" output.warn
  format.volume output
  format.number.if.use.for.article output
  format.pages output
  format.date "year" output.warn
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {book}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  author empty$
    { format.editors "author and editor" output.warn }
    { format.authors output.nonnull }
  if$
  name.or.dash
  format.book.title.edition output
  format.series output
  author empty$
    { skip$ }
    { format.editors output }
  if$
  format.address.publisher.date output
  format.volume output
  format.number output
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {booklet}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors output
  name.or.dash
  format.article.title "title" output.warn
  format.howpublished "howpublished" bibinfo.check output
  format.organization "organization" bibinfo.check output
  format.address "address" bibinfo.check output
  format.date output
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {electronic}
{ std.status.using.period
  start.entry
  if.url.alt.interword.spacing
  format.authors output
  name.or.dash
  format.date.electronic output
  format.article.title.electronic output
  format.howpublished "howpublished" bibinfo.check output
  format.organization "organization" bibinfo.check output
  format.address "address" bibinfo.check output
  format.note output
  format.url output
  fin.entry
  empty.entry.warn
  if.url.std.interword.spacing
}

FUNCTION {inbook}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  author empty$
    { format.editors "author and editor" output.warn }
    { format.authors output.nonnull }
  if$
  name.or.dash
  format.book.title.edition output
  format.series output
  format.address.publisher.date output
  format.volume output
  format.number output
  format.chapter output
  format.pages output
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {incollection}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors "author" output.warn
  name.or.dash
  format.article.title "title" output.warn
  format.in.booktitle.edition "booktitle" output.warn
  format.series output
  format.editors output
  format.address.publisher.date.nowarn output
  format.volume output
  format.number output
  format.chapter output
  format.pages output
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {inproceedings}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors "author" output.warn
  name.or.dash
  format.article.title "title" output.warn
  format.in.booktitle "booktitle" output.warn
  format.series output
  format.editors output
  format.volume output
  format.number output
  publisher empty$
    { format.address.organization.date output }
    { format.organization "organization" bibinfo.check output
      format.address.publisher.date output
    }
  if$
  format.paper output
  format.pages output
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {manual}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors output
  name.or.dash
  format.book.title.edition "title" output.warn
  format.howpublished "howpublished" bibinfo.check output 
  format.organization "organization" bibinfo.check output
  format.address "address" bibinfo.check output
  format.date output
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {mastersthesis}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors "author" output.warn
  name.or.dash
  format.article.title "title" output.warn
  format.master.thesis.type output.nonnull
  format.school "school" bibinfo.warn output
  format.address "address" bibinfo.check output
  format.date "year" output.warn
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {misc}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors output
  name.or.dash
  format.article.title output
  format.howpublished "howpublished" bibinfo.check output 
  format.organization "organization" bibinfo.check output
  format.address "address" bibinfo.check output
  format.pages output
  format.date output
  format.note output
  format.url output
  fin.entry
  empty.entry.warn
  if.url.std.interword.spacing
}

FUNCTION {patent}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors output
  name.or.dash
  format.article.title output
  format.patent.nationality.type.number output
  format.patent.date output
  format.note output
  format.url output
  fin.entry
  empty.entry.warn
  if.url.std.interword.spacing
}

FUNCTION {periodical}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.editors output
  name.or.dash
  format.book.title "title" output.warn
  format.series output
  format.volume output
  format.number output
  format.organization "organization" bibinfo.check output
  format.date "year" output.warn
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {phdthesis}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors "author" output.warn
  name.or.dash
  format.article.title "title" output.warn
  format.phd.thesis.type output.nonnull
  format.school "school" bibinfo.warn output
  format.address "address" bibinfo.check output
  format.date "year" output.warn
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {proceedings}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.editors output
  name.or.dash
  format.book.title "title" output.warn
  format.series output
  format.volume output
  format.number output
  publisher empty$
    { format.address.organization.date output }
    { format.organization "organization" bibinfo.check output
      format.address.publisher.date output
    }
  if$
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {standard}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors output
  name.or.dash
  format.book.title "title" output.warn
  format.howpublished "howpublished" bibinfo.check output 
  format.organization.institution.standard.type.number output
  format.revision output
  format.date output
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {techreport}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors "author" output.warn
  name.or.dash
  format.article.title "title" output.warn
  format.howpublished "howpublished" bibinfo.check output 
  format.institution "institution" bibinfo.warn output
  format.address "address" bibinfo.check output
  format.tech.report.number output.nonnull
  format.date "year" output.warn
  format.note output
  format.url output
  fin.entry
  if.url.std.interword.spacing
}

FUNCTION {unpublished}
{ std.status.using.comma
  start.entry
  if.url.alt.interword.spacing
  format.authors "author" output.warn
  name.or.dash
  format.article.title "title" output.warn
  format.date output
  format.note "note" output.warn
  format.url output
  fin.entry
  if.url.std.interword.spacing
}


% The special entry type which provides the user interface to the
% BST controls
FUNCTION {IEEEtranBSTCTL}
{ is.print.banners.to.terminal
    { "** IEEEtran BST control entry " quote$ * cite$ * quote$ * " detected." *
      top$
    }
    { skip$ }
  if$
  CTLuse_article_number
  empty$
    { skip$ }
    { CTLuse_article_number
      yes.no.to.int
      'is.use.number.for.article :=
    }
  if$
  CTLuse_paper
  empty$
    { skip$ }
    { CTLuse_paper
      yes.no.to.int
      'is.use.paper :=
    }
  if$
  CTLuse_forced_etal
  empty$
    { skip$ }
    { CTLuse_forced_etal
      yes.no.to.int
      'is.forced.et.al :=
    }
  if$
  CTLmax_names_forced_etal
  empty$
    { skip$ }
    { CTLmax_names_forced_etal
      string.to.integer
      'max.num.names.before.forced.et.al :=
    }
  if$
  CTLnames_show_etal
  empty$
    { skip$ }
    { CTLnames_show_etal
      string.to.integer
      'num.names.shown.with.forced.et.al :=
    }
  if$
  CTLuse_alt_spacing
  empty$
    { skip$ }
    { CTLuse_alt_spacing
      yes.no.to.int
      'is.use.alt.interword.spacing :=
    }
  if$
  CTLalt_stretch_factor
  empty$
    { skip$ }
    { CTLalt_stretch_factor
      'ALTinterwordstretchfactor :=
      "\renewcommand{\BIBentryALTinterwordstretchfactor}{"
      ALTinterwordstretchfactor * "}" *
      write$ newline$
    }
  if$
  CTLdash_repeated_names
  empty$
    { skip$ }
    { CTLdash_repeated_names
      yes.no.to.int
      'is.dash.repeated.names :=
    }
  if$
  CTLname_format_string
  empty$
    { skip$ }
    { CTLname_format_string
      'name.format.string :=
    }
  if$
  CTLname_latex_cmd
  empty$
    { skip$ }
    { CTLname_latex_cmd
      'name.latex.cmd :=
    }
  if$
  CTLname_url_prefix
  missing$
    { skip$ }
    { CTLname_url_prefix
      'name.url.prefix :=
    }
  if$


  num.names.shown.with.forced.et.al max.num.names.before.forced.et.al >
    { "CTLnames_show_etal cannot be greater than CTLmax_names_forced_etal in " cite$ * warning$ 
      max.num.names.before.forced.et.al 'num.names.shown.with.forced.et.al :=
    }
    { skip$ }
  if$
}


%%%%%%%%%%%%%%%%%%%
%% ENTRY ALIASES %%
%%%%%%%%%%%%%%%%%%%
FUNCTION {conference}{inproceedings}
FUNCTION {online}{electronic}
FUNCTION {internet}{electronic}
FUNCTION {webpage}{electronic}
FUNCTION {www}{electronic}
FUNCTION {default.type}{misc}



%%%%%%%%%%%%%%%%%%
%% MAIN PROGRAM %%
%%%%%%%%%%%%%%%%%%

READ

EXECUTE {initialize.controls}
EXECUTE {initialize.status.constants}
EXECUTE {banner.message}

EXECUTE {initialize.longest.label}
ITERATE {longest.label.pass}

EXECUTE {begin.bib}
ITERATE {call.type$}
EXECUTE {end.bib}

EXECUTE{completed.message}


%% That's all folks, mds.
