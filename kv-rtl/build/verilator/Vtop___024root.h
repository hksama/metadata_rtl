// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Design internal header
// See Vtop.h for the primary calling header

#ifndef VERILATED_VTOP___024ROOT_H_
#define VERILATED_VTOP___024ROOT_H_  // guard

#include "verilated.h"


class Vtop__Syms;

class alignas(VL_CACHE_LINE_BYTES) Vtop___024root final : public VerilatedModule {
  public:

    // DESIGN SPECIFIC STATE
    VL_IN8(clk,0,0);
    VL_IN8(rst,0,0);
    VL_IN8(event_valid,0,0);
    VL_IN8(operation,2,0);
    VL_OUT8(resp_valid,0,0);
    VL_OUT8(resp_error,0,0);
    VL_OUT8(meta_valid,0,0);
    VL_OUT8(meta_refcount,7,0);
    VL_OUT8(meta_state,1,0);
    CData/*0:0*/ top__DOT__u_tracker__DOT__ev_pending;
    CData/*2:0*/ top__DOT__u_tracker__DOT__ev_operation;
    CData/*0:0*/ top__DOT__u_tracker__DOT__page_valid;
    CData/*7:0*/ top__DOT__u_tracker__DOT__page_refcount;
    CData/*1:0*/ top__DOT__u_tracker__DOT__page_state;
    CData/*0:0*/ top__DOT__u_tracker__DOT__next_error;
    CData/*0:0*/ top__DOT__u_tracker__DOT__next_page_valid;
    CData/*7:0*/ top__DOT__u_tracker__DOT__next_page_refcount;
    CData/*1:0*/ top__DOT__u_tracker__DOT__next_page_state;
    CData/*0:0*/ __Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__Vfuncout;
    CData/*7:0*/ __Vfunc_sat_dec_ref__2__Vfuncout;
    CData/*7:0*/ __Vfunc_sat_dec_ref__2__value;
    CData/*0:0*/ __Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__Vfuncout;
    CData/*0:0*/ __VstlFirstIteration;
    CData/*0:0*/ __Vtrigprevexpr___TOP__clk__0;
    CData/*0:0*/ __VactContinue;
    VL_IN16(request_id,15,0);
    VL_IN16(page_id,15,0);
    VL_IN16(event_timestamp,15,0);
    VL_OUT16(meta_page_id,15,0);
    VL_OUT16(meta_request_id,15,0);
    VL_OUT16(meta_access_count,15,0);
    VL_OUT16(meta_last_access,15,0);
    VL_OUT16(hw_timestamp,15,0);
    SData/*15:0*/ top__DOT__u_tracker__DOT__ev_request_id;
    SData/*15:0*/ top__DOT__u_tracker__DOT__ev_page_id;
    SData/*15:0*/ top__DOT__u_tracker__DOT__ev_timestamp;
    SData/*15:0*/ top__DOT__u_tracker__DOT__page_page_id;
    SData/*15:0*/ top__DOT__u_tracker__DOT__page_request_id;
    SData/*15:0*/ top__DOT__u_tracker__DOT__page_access_count;
    SData/*15:0*/ top__DOT__u_tracker__DOT__page_last_access;
    SData/*15:0*/ top__DOT__u_tracker__DOT__next_page_page_id;
    SData/*15:0*/ top__DOT__u_tracker__DOT__next_page_request_id;
    SData/*15:0*/ top__DOT__u_tracker__DOT__next_page_access_count;
    SData/*15:0*/ top__DOT__u_tracker__DOT__next_page_last_access;
    SData/*15:0*/ __Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__stored_id;
    SData/*15:0*/ __Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__event_id;
    SData/*15:0*/ __Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__stored_id;
    SData/*15:0*/ __Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__event_id;
    SData/*15:0*/ __Vfunc_sat_inc_u__4__Vfuncout;
    SData/*15:0*/ __Vfunc_sat_inc_u__4__value;
    IData/*31:0*/ __VactIterCount;
    VlUnpacked<CData/*0:0*/, 2> __Vm_traceActivity;
    VlTriggerVec<1> __VstlTriggered;
    VlTriggerVec<1> __VactTriggered;
    VlTriggerVec<1> __VnbaTriggered;

    // INTERNAL VARIABLES
    Vtop__Syms* const vlSymsp;

    // CONSTRUCTORS
    Vtop___024root(Vtop__Syms* symsp, const char* v__name);
    ~Vtop___024root();
    VL_UNCOPYABLE(Vtop___024root);

    // INTERNAL METHODS
    void __Vconfigure(bool first);
};


#endif  // guard
